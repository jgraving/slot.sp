#!/usr/bin/env python3
"""Native packaged-server test. No user networks or credentials are used."""
import os, pathlib, signal, socket, subprocess, sys, tempfile, time, shutil
runtime=pathlib.Path(sys.argv[1]).resolve()
script=pathlib.Path(__file__).resolve().parent/'System/slotsmb.sh'
with tempfile.TemporaryDirectory() as tmp:
    root=pathlib.Path(tmp); (root/'System').mkdir();shutil.copytree(runtime,root/'System/samba')
    (root/'Config').mkdir();(root/'Config/secret').write_text('PRIVATE CARD CONTENT')
    run=root/'run';run.mkdir()
    loader=next((root/'System/samba/lib').glob('ld-linux-*.so.*'))
    env=dict(os.environ,AGS_RUN=str(run),SLOT_SMB_LOADER=str(loader),SLOT_SMB_TEST_PORT='1445',SLOT_SMB_TEST_INTERFACE='lo',GCONV_PATH=str(root/'System/samba/gconv'))
    password='ABCDEFGH23456789'
    subprocess.run(['sh',str(script),'prepare',str(root),'127.0.0.1'],input=password+'\n',env=env,text=True,check=True)
    config=(run/'slot-smb/smb.conf').read_text()
    assert 'SMB3_00' in config and 'server smb encrypt = required' in config
    assert password not in ''.join(p.read_text(errors='replace') for p in (run/'slot-smb').rglob('*') if p.is_file())
    auth=root/'auth';auth.write_text(f'username = slot\npassword = {password}\n');auth.chmod(0o600)
    server=subprocess.Popen(['sh',str(script),'serve',str(root),str(os.getpid())],env=env,start_new_session=True,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
    def opened():
        try:
            with socket.create_connection(('127.0.0.1',1445),.1):return True
        except OSError:return False
    def client(command,ok=True,protection='encrypt',authfile=auth):
        args=[str(loader),'--library-path',str(root/'System/samba/lib'),str(root/'System/samba/bin/smbclient'),'//127.0.0.1/Games','-p','1445','-m','SMB3',f'--client-protection={protection}','-A',str(authfile),'-c',command]
        result=subprocess.run(args,text=True,capture_output=True,timeout=15,env=env)
        if ok:assert result.returncode==0,(result.stdout,result.stderr)
        else:assert result.returncode!=0,(result.stdout,result.stderr)
        return result.stdout+result.stderr
    try:
        deadline=time.monotonic()+10
        while not opened():
            assert server.poll() is None,server.stderr.read().decode()
            assert time.monotonic()<deadline,'server did not listen'
            time.sleep(.1)
        # Finder and Explorer enumerate shares through named-pipe SRVSVC, not just direct file access.
        enumerate_args=[str(loader),'--library-path',str(root/'System/samba/lib'),str(root/'System/samba/bin/smbclient'),'-L','127.0.0.1','-p','1445','-m','SMB3','--client-protection=encrypt','-A',str(auth)]
        enumeration=subprocess.run(enumerate_args,text=True,capture_output=True,timeout=15,env=env)
        assert enumeration.returncode==0 and 'Games' in enumeration.stdout,(enumeration.stdout,enumeration.stderr)
        listing=client('ls');assert all(p in listing for p in ('GB','GBC','GBA','Wallpapers'));assert 'Config' not in listing and 'secret' not in listing
        bad=root/'bad-auth';bad.write_text('username = slot\npassword = WRONG\n');bad.chmod(0o600)
        client('ls',False,authfile=bad)
        client('ls',False,protection='off')
        source=root/'upload.gba';source.write_bytes(bytes(1024))
        client(f'put {source} GBA/Uploaded.gba')
        assert (root/'Inbox/GBA/Uploaded.gba').read_bytes()==source.read_bytes()
        wallpaper=root/'wallpaper.png'
        wallpaper.write_bytes((script.parents[2]/'assets/background_default.png').read_bytes())
        client(f'put {wallpaper} Wallpapers/Uploaded.png')
        assert (root/'Inbox/Wallpapers/Uploaded.png').read_bytes()==wallpaper.read_bytes()
        (root/'Inbox/private-link').symlink_to(root/'Config/secret')
        client(f'get private-link {root}/leaked',False)
        client(f'get ../Config/secret {root}/leaked',False)
        assert not (root/'leaked').exists() or not (root/'leaked').read_bytes()
    finally:
        subprocess.run(['sh',str(script),'stop'],env=env,check=True)
        try:server.wait(timeout=5)
        except subprocess.TimeoutExpired:
            os.killpg(server.pid,signal.SIGKILL);server.wait()
            raise AssertionError('production stop helper did not terminate the SMB process group')
    assert not opened(),'SMB listener survived shutdown'
    assert not (run/'slot-smb/smbpasswd').exists(),'runtime password database survived shutdown'
    # A launcher crash must also close the server without needing the stop helper.
    subprocess.run(['sh',str(script),'prepare',str(root),'127.0.0.1'],input=password+'\n',env=env,text=True,check=True)
    parent=subprocess.Popen(['sleep','30'])
    server=subprocess.Popen(['sh',str(script),'serve',str(root),str(parent.pid)],env=env,start_new_session=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    try:
        deadline=time.monotonic()+10
        while not opened():
            assert server.poll() is None,'watchdog test server failed'
            assert time.monotonic()<deadline
            time.sleep(.1)
        parent.terminate();parent.wait(timeout=3)
        try:server.wait(timeout=5)
        except subprocess.TimeoutExpired:
            raise AssertionError('launcher-crash watchdog left the server running')
        assert not opened(),'SMB listener survived launcher crash'
    finally:
        if parent.poll() is None:parent.terminate();parent.wait()
        if server.poll() is None:os.killpg(server.pid,signal.SIGKILL);server.wait()
print('Real SMB3: password, encryption, upload isolation and shutdown checks passed')
