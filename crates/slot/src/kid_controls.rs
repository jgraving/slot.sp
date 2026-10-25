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
        self.kid_menu = Some(crate::kid_menu::Menu::new(root, self.kids.clone()));
    }

    pub(super) fn kid_input(&mut self, action: Action) -> bool {
        if matches!(
            action,
            Action::LidClose
                | Action::LidOpen
                | Action::PowerPress
                | Action::PowerTap
                | Action::PowerHold
                | Action::PowerOff
        ) {
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
            let close = !menu.visible;
            if let Some(policy) = changed {
                self.stop_for_parent();
                self.kids = policy;
                self.kid_error = false;
                self.apply_kid_scope();
            }
            if close {
                self.kid_menu = None;
            }
            return true;
        }
        if self.kids.active {
            if matches!(action, Action::QuickMenu | Action::GameMenu)
                || (self.on_shelf() && action == Action::GbaDown(Btn::Start))
            {
                self.open_kid_controls();
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
        let Some(cart) = self.shelf().carts.get(self.shelf().index) else {
            return;
        };
        let mut next = self.kids.clone();
        next.toggle(cart);
        if next.save(root).is_err() {
            self.hud.toast(Toast::CardUnwritable, self.now());
            return;
        }
        self.kids = next;
        self.mark_kid_approved();
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
        if !result.unwrap_or(false)
            && self.kid_menu.is_none()
            && !matches!(self.phase, Phase::Doze { .. })
        {
            self.stop_for_parent();
            self.apply_kid_scope();
            self.kid_menu = Some(crate::kid_menu::Menu::notice(
                root,
                self.kids.clone(),
                if self.kid_error {
                    "Ask a parent to check the settings"
                } else {
                    "Ask a parent for more play time"
                },
            ));
        }
    }

    pub fn set_kid_face(&mut self, face: TexId, w: u32, h: u32) {
        for (_, shelf) in &mut self.shelves {
            shelf.set_kid_face(face, w, h);
        }
    }
}
