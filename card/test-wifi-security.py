"""Check the device boundary never writes network secrets or passes them to a shell."""
from pathlib import Path
root = Path(__file__).resolve().parent
script = (root / 'System/slotwifi.sh').read_text()
source = (root.parent / 'crates/slot/src/wifi.rs').read_text()
assert "update_config=0" in script
assert "update_config=1" not in script
assert 'conf="$RUN/slotwifi.conf"' in script
assert 'OWNER="$RUN/slotwifi.owner"' in script
assert 'Self::ok(&ctrl, "SAVE_CONFIG")' not in source
assert 'format!("{cmd}: {e}")' not in source
assert 'set("psk", &psk_value(psk)' in source
print('Wi-Fi credential boundary checks passed')
