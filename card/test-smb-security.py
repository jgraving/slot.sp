#!/usr/bin/env python3
"""Exercise the production config generator without opening any network sockets."""
import os,pathlib,subprocess,tempfile
script=pathlib.Path(__file__).resolve().parent/'System/slotsmb.sh'
with tempfile.TemporaryDirectory() as d:
    root=pathlib.Path(d);bin=root/'System/samba/bin';bin.mkdir(parents=True);run=root/'run';run.mkdir()
    loader=root/'loader';loader.write_text('#!/bin/sh\nshift 2\nexec "$@"\n');loader.chmod(0o755)
    for name in ('smbd','wsdd2','smbpasswd','testparm','samba-dcerpcd','rpcd_classic'):
        p=bin/name;p.write_text('#!/bin/sh\ncat >/dev/null\n');p.chmod(0o755)
    env=dict(os.environ,AGS_RUN=str(run),SLOT_SMB_LOADER=str(loader))
    def prepare(password='ABCDEFGH23456789',ip='192.168.1.2'):
        return subprocess.run(['sh',str(script),'prepare',str(root),ip],input=password+'\n',text=True,env=env,capture_output=True)
    assert prepare().returncode==0
    cfg=(run/'slot-smb/smb.conf').read_text()
    for line in ['server min protocol = SMB3_00','server smb encrypt = required','server signing = mandatory','guest ok = no','follow symlinks = no','wide links = no','bind interfaces only = yes','interfaces = 192.168.1.2/32','valid users = nobody',f'path = {root}/Inbox']:
        assert line in cfg,line
    assert str(root/'Config') not in cfg and str(root/'System') not in cfg
    assert 'ABCDEFGH23456789' not in cfg
    assert (run/'slot-smb/smb.conf').stat().st_mode&0o077==0
    assert prepare('weak').returncode!=0
    assert prepare(ip='192.168.1.2\n[evil]').returncode!=0
    (root/'Inbox/GBA').rmdir();(root/'Inbox/GBA').symlink_to(root/'System',target_is_directory=True)
    assert prepare().returncode!=0,'followed an Inbox directory link'
print('SMB configuration/password/path restrictions passed')
