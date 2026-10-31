use super::*;

impl App {
    pub fn kid_mode(&self) -> bool {
        self.kids.active
    }
    pub fn parent_controls_open(&self) -> bool {
        self.kid_menu.is_some()
    }
    pub fn kid_volume_limit(&self) -> u8 {
        if self.kids.active {
            self.kids.volume_limit
        } else {
            100
        }
    }

    pub fn kid_action_allowed(&self, action: Action) -> bool {
        if !self.kids.active {
            return true;
        }
        matches!(
            action,
            Action::GbaDown(_)
                | Action::GbaUp(_)
                | Action::ShelfLeft
                | Action::ShelfRight
                | Action::Insert
                | Action::Eject
                | Action::VolumeUp
                | Action::VolumeDown
                | Action::MuteToggle
                | Action::BrightnessUp
                | Action::BrightnessDown
                | Action::QuickMenu
                | Action::GameMenu
                | Action::PreviousShelf
                | Action::NextShelf
                | Action::PowerPress
                | Action::PowerTap
                | Action::PowerHold
                | Action::PowerOff
                | Action::LidClose
                | Action::LidOpen
                | Action::FfStop
                | Action::RewindStop
        )
    }

    pub(super) fn open_kid_controls(&mut self) {
        let Some(root) = self.root.clone() else {
            return;
        };
        self.play_held = None;
        self.held_dir = None;
        self.shelf_mut().release_hold();
        let mut menu = crate::kid_menu::Menu::new(root, self.kids.clone());
        menu.set_remaining(self.kids.active.then(|| (self.kids.remaining(self.wall_secs()), self.kids.time_budget())));
        self.kid_menu = Some(menu);
    }

    pub(super) fn kid_input(&mut self, action: Action) -> bool {
        match action {
            Action::GbaUp(Btn::X) => {
                if let Some(pending) = &mut self.kid_removal { pending.released = true; }
            }
            Action::GbaDown(Btn::X) | Action::GbaUp(_) => {}
            _ => self.kid_removal = None,
        }
        if matches!(
            action,
            Action::LidClose
                | Action::LidOpen
                | Action::PowerPress
                | Action::PowerTap
                | Action::PowerHold
                | Action::PowerOff
        ) {
            if let Some(menu) = &mut self.kid_menu { menu.stop_repeat(); }
            return false;
        }
        if matches!(self.phase, Phase::Doze { .. }) {
            return false;
        }
        if self.kid_menu.is_some() {
            let wall = self.wall_secs();
            let menu = self.kid_menu.as_mut().unwrap();
            menu.input(action, wall);
            let changed = menu.changed.take();
            let child_action = menu.child_action.take();
            let close = !menu.visible;
            if let Some(policy) = changed {
                let scope_changed = self.kids.active != policy.active || self.kids.approved != policy.approved;
                self.kids = policy;
                self.kid_error = false;
                // Ordinary parent adjustments must not eject an approved game.
                // Stop only if the new policy actually disallows this session.
                let blocked = self.kids.active && (
                    self.kids.remaining(self.wall_secs()) == 0
                    || match &self.phase {
                        Phase::Playing { cart } | Phase::Inserting { cart, .. } => {
                            !self.catalog.iter().any(|c| c.platform == self.platform
                                && c.stem == *cart && self.kids.allows(c))
                        }
                        _ => false,
                    });
                if blocked { self.stop_for_parent(); }
                if scope_changed { self.apply_kid_scope(); }
            }
            if close { self.kid_menu = None; }
            if let Some(action) = child_action {
                if action == Action::Eject { self.apply(action); } else { self.adjust(action); }
            }
            let volume = self.volume();
            let brightness = (u16::from(self.brightness()) * 100 / u16::from(BRIGHTNESS_MAX)) as u8;
            let time_left = self.kids.active.then(|| (self.kids.remaining(self.wall_secs()), self.kids.time_budget()));
            if let Some(menu) = &mut self.kid_menu {
                menu.set_remaining(time_left);
                menu.child_volume = volume;
                menu.child_brightness = brightness;
            }
            return true;
        }
        if self.kids.active {
            if matches!(action, Action::QuickMenu | Action::GameMenu)
                || (self.on_shelf() && action == Action::GbaDown(Btn::Start))
            {
                if let Some(root) = self.root.clone() {
                    let mut menu = crate::kid_menu::Menu::child(root, self.kids.clone(),
                        matches!(self.phase, Phase::Playing { .. }), self.volume(),
                        (u16::from(self.brightness()) * 100 / u16::from(BRIGHTNESS_MAX)) as u8);
                    menu.set_reference(self.reference_pages(matches!(self.phase, Phase::Playing { .. })));
                    menu.set_remaining(self.kids.active.then(|| (self.kids.remaining(self.wall_secs()), self.kids.time_budget())));
                    self.kid_surface.snap(matches!(self.phase,Phase::Playing{..}));
                    self.kid_menu = Some(menu);
                }
                return true;
            }
            if !self.kid_action_allowed(action) {
                return true;
            }
            if self.on_shelf() && matches!(action, Action::GbaDown(Btn::X | Btn::Y)) {
                return true;
            }
        } else if self.on_shelf()
            && action == Action::GbaDown(Btn::X)
            && self.bluetooth.is_none()
            && self.core_picker.is_none()
        {
            self.toggle_kid_approved();
            return true;
        }
        false
    }

    pub(super) fn stop_for_parent(&mut self) {
        self.close_polaroids();
        if let Phase::Playing { cart } | Phase::Inserting { cart, .. } = &self.phase {
            let cart = cart.clone();
            self.end_link();
            self.close_game_menu();
            self.flush_resume();
            self.flush_eject(&cart);
        }
        self.phase = Phase::Shelf;
        self.in_game_menu = None;
        self.footer_row = None;
        self.reset_prompt = None;
        self.delete_prompt = false;
        self.repair_prompt = false;
        self.play_held = None;
        self.held_dir = None;
        self.state.cart = None;
        self.state.cart_platform = None;
        self.persist();
    }

    pub(super) fn apply_kid_scope(&mut self) {
        // Keep the same racks and their full persistent memberships. Only their
        // visible contents change, so parent favorites/recents are never erased.
        for at in 0..self.shelves.len() {
            if let Some(platform) = self.shelves[at].0.platform() {
                let carts = self
                    .catalog
                    .iter()
                    .filter(|c| {
                        c.platform == platform && (!self.kids.active || self.kids.allows(c))
                    })
                    .cloned()
                    .collect();
                self.restock(at, carts);
            }
        }
        self.restock_recents();
        self.restock_favorites();
        self.mark_kid_approved();
        if self.shelf().carts.is_empty() {
            self.shelf_at = self.next_shelf(1).unwrap_or(0);
        }
        self.name_pending = true;
        self.shelf_named = None;
    }

    pub(super) fn mark_kid_approved(&mut self) {
        let approved = &self.kids.approved;
        for (_, shelf) in &mut self.shelves {
            shelf.mark_kids(|c| approved.contains(&(c.platform, c.stem.clone())));
        }
    }

    pub(super) fn toggle_kid_approved(&mut self) {
        if self.kids.active {
            return;
        }
        let Some(root) = self.root.as_ref() else {
            return;
        };
        let Some(cart) = self.shelf().carts.get(self.shelf().index).cloned() else {
            return;
        };
        if self.kids.allows(&cart) {
            if self.kid_removal_pending() {
                if !self.kid_removal.as_ref().is_some_and(|p| p.released) { return; }
            } else {
                self.kid_removal = Some(FavoriteRemoval {
                    platform: cart.platform, stem: cart.stem.clone(), shelf: self.shelf_at,
                    since: self.now(), released: false,
                });
                return;
            }
        }
        self.kid_removal = None;
        let mut next = self.kids.clone();
        next.toggle(&cart);
        if next.save(root).is_err() {
            self.hud.toast(Toast::CardUnwritable, self.now());
            return;
        }
        self.kids = next;
        self.mark_kid_approved();
    }

    pub fn kid_removal_pending(&self) -> bool {
        self.kid_removal.as_ref().is_some_and(|p| {
            !self.kids.active && self.kid_menu.is_none()
                && matches!(self.phase, Phase::Shelf) && self.core_picker.is_none()
                && p.shelf == self.shelf_at
                && self.now().saturating_sub(p.since) < slot_ui::FAVORITE_REMOVE_MS
                && self.shelf().carts.get(self.shelf().index)
                    .is_some_and(|c| c.platform == p.platform && c.stem == p.stem)
        })
    }
    pub fn set_kid_remove_face(&mut self, tex: TexId, w: u32, h: u32) {
        self.kid_remove_face = Some((tex, w, h));
    }
    pub(super) fn draw_kid_removal(&self, out: &mut Vec<Draw>) {
        if !self.kid_removal_pending() { return; }
        let (Some(pending), Some(face)) = (&self.kid_removal, self.kid_remove_face) else { return; };
        let (cw, ch) = slot_ui::cart_box(pending.platform);
        let (x, scale) = self.shelf().selected_at(self.canvas());
        let (w, h) = (cw as f32 * scale, ch as f32 * scale);
        let body = slot_ui::Placed { x, y: slot_ui::foot_y(ch as f32) - h, w, h };
        slot_ui::draw_favorite_remove(self.canvas(), body, face,
            self.now().saturating_sub(pending.since), out);
    }

    pub(super) fn kid_can_launch(&mut self) -> bool {
        if !self.kids.active {
            return true;
        }
        let allowed = !self.kid_error
            && self.kids.remaining(self.wall_secs()) > 0
            && self
                .shelf()
                .carts
                .get(self.shelf().index)
                .is_some_and(|c| self.kids.allows(c));
        if !allowed {
            self.open_kid_controls();
        }
        allowed
    }

    pub(super) fn tick_kid_policy(&mut self) {
        if !self.kids.active {
            return;
        }
        let Some(root) = self.root.clone() else {
            return;
        };
        let wall = self.wall_secs();
        let playing = matches!(self.phase, Phase::Playing { .. })
            && self.kid_menu.is_none()
            && self.in_game_menu.is_none()
            && self.game_menu.is_none()
            && !self.shutting_down();
        let result = if self.kid_error {
            Err(std::io::Error::other("Parent settings need attention"))
        } else {
            self.kids.tick(&root, wall, self.now(), playing)
        };
        if result.as_ref().is_err() {
            self.kid_error = true;
        }
        let allowed = result.unwrap_or(false);
        let remaining = self.kids.remaining(wall);
        let budget = self.kids.time_budget();
        if let Some(menu) = &mut self.kid_menu {
            menu.set_remaining(Some((remaining, budget)));
        }
        let awake = !matches!(self.phase, Phase::Doze { .. }) && !self.shutting_down();
        let child_overlay = self.kid_menu.as_ref().is_some_and(|m| m.child_menu() || m.time_warning());
        if !allowed && awake && (self.kid_menu.is_none() || child_overlay) {
            self.show_kid_time_end(root);
            return;
        }
        let warning_open = self.kid_menu.as_ref().is_some_and(|m| m.time_warning());
        if !allowed || !awake || (!playing && !warning_open) { return; }
        match self.kids.take_warning(&root, wall) {
            Ok(Some(seconds)) if !warning_open => {
                // Enter the pause immediately; no gameplay credit is charged
                // for time spent reading the warning, even without another frame.
                if self.kids.tick(&root, wall, self.now(), false).is_err() {
                    self.kid_error = true;
                    self.show_kid_time_end(root);
                    return;
                }
                self.kid_menu = Some(crate::kid_menu::Menu::warning(root, self.kids.clone(), seconds));
            }
            Err(_) => {
                self.kid_error = true;
                self.show_kid_time_end(root);
            }
            _ => {}
        }
    }

    fn show_kid_time_end(&mut self, root: std::path::PathBuf) {
        self.stop_for_parent();
        self.apply_kid_scope();
        let menu = crate::kid_menu::Menu::notice(
            root, self.kids.clone(),
            if self.kid_error { "Ask a parent to check the settings" }
            else { "Ask a parent for more play time" },
        );
        self.kid_menu = Some(menu);
    }

    pub fn set_kid_face(&mut self, face: TexId, w: u32, h: u32) {
        for (_, shelf) in &mut self.shelves {
            shelf.set_kid_face(face, w, h);
        }
    }
}
