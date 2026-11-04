#!/usr/bin/env python3
"""Native packaged-server test. No user networks or credentials are used."""
import os, pathlib, signal, socket, subprocess, sys, tempfile, time, shutil, select, threading

class EncryptionProbe:
    """Transparent local relay: observe SMB framing without knowing session keys."""
    def __enter__(self):
        self.encrypted=0;self.plain_file_commands=[];self.closed=threading.Event();self.workers=[]
        self.listener=socket.socket();self.listener.bind(('127.0.0.1',0));self.listener.listen();self.listener.settimeout(.2)
        self.port=self.listener.getsockname()[1]
        def accept():
            while not self.closed.is_set():
                try:client,_=self.listener.accept()
                except socket.timeout:continue
                except OSError:break
                worker=threading.Thread(target=self.relay,args=(client,),daemon=True);self.workers.append(worker);worker.start()
        self.thread=threading.Thread(target=accept,daemon=True);self.thread.start();return self
    def relay(self,client):
        server=None;pending=b''
        try:
            server=socket.create_connection(('127.0.0.1',1445),2)
            while not self.closed.is_set():
                ready,_,_=select.select([client,server],[],[],.2)
                for source in ready:
                    data=source.recv(65536)
                    if not data:return
                    if source is client:
                        pending+=data
                        while len(pending)>=4:
                            size=int.from_bytes(pending[1:4],'big')
                            if len(pending)<size+4:break
                            frame=pending[4:size+4];pending=pending[size+4:]
                            if frame[:4]==b'\xfdSMB':self.encrypted+=1
                            if frame[:4]==b'\xfeSMB' and len(frame)>=64:
                                command=int.from_bytes(frame[12:14],'little')
                                if command in (5,8,9,14):self.plain_file_commands.append(command)
                    (server if source is client else client).sendall(data)
        except OSError:pass
        finally:
            client.close()
            if server is not None:server.close()
    def __exit__(self,*exc):
        self.closed.set();self.listener.close();self.thread.join(timeout=1)
        for worker in self.workers:worker.join(timeout=1)

runtime=pathlib.Path(sys.argv[1]).resolve()
script=pathlib.Path(__file__).resolve().parent/'System/slotsmb.sh'
with tempfile.TemporaryDirectory() as tmp:
    root=pathlib.Path(tmp);root.chmod(0o755) # Match the traversable SD root for unprivileged runtime conversion modules.
    (root/'System').mkdir();shutil.copytree(runtime,root/'System/samba')
    (root/'Config').mkdir();(root/'Config/secret').write_text('PRIVATE CARD CONTENT')
    run=root/'run';run.mkdir()
    loader=next((root/'System/samba/lib').glob('ld-linux-*.so.*'))
    env=dict(os.environ,AGS_RUN=str(run),SLOT_SMB_LOADER=str(loader),SLOT_SMB_TEST_PORT='1445',SLOT_SMB_TEST_INTERFACE='lo',GCONV_PATH=str(root/'System/samba/gconv'))
    # Model a content card whose mount drops execute permissions. The test
    # client still needs the original loader, so use a non-executable server copy.
    card_loader=root/'System/samba/lib/card-loader'
    shutil.copyfile(loader,card_loader)
    card_loader.chmod(0o644)
    env['SLOT_SMB_LOADER']=str(card_loader)
    for binary in (root/'System/samba/bin').iterdir():
        if binary.name != 'smbclient':
            binary.chmod(0o644)
    for wrapper in (root/'System/samba/libexec').iterdir():
        wrapper.chmod(0o644)
    password='ABCDEFGH23456789'
    subprocess.run(['sh',str(script),'prepare',str(root),'127.0.0.1'],input=password+'\n',env=env,text=True,check=True)
    config=(run/'slot-smb/smb.conf').read_text()
    assert 'SMB3_00' in config and 'server smb encrypt = required' in config
    assert password not in ''.join(p.read_text(errors='replace') for p in (run/'slot-smb').rglob('*') if p.is_file())
    # Test-only diagnostics contain only this disposable fixture's account/session.
    (run/'slot-smb/smb.conf').write_text(config.replace('log file = /dev/null',f'log file = {run}/slot-smb/test-log.%m').replace('log level = 0','log level = 3'))
    auth=root/'auth';auth.write_text(f'username = slot\npassword = {password}\n');auth.chmod(0o600)
    server=subprocess.Popen(['sh',str(script),'serve',str(root),str(os.getpid())],env=env,start_new_session=True,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
    def opened():
        try:
            with socket.create_connection(('127.0.0.1',1445),.1):return True
        except OSError:return False
    def client(command,ok=True,protection='encrypt',authfile=auth,port=1445,protocol='SMB3',share='Games'):
        args=[str(loader),'--library-path',str(root/'System/samba/lib'),str(root/'System/samba/bin/smbclient'),f'//127.0.0.1/{share}','-p',str(port),'-m',protocol,f'--client-protection={protection}','-A',str(authfile),'-c',command]
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
        assert enumeration.returncode==0 and all(name in enumeration.stdout for name in ('Games','Labels','Wallpapers')),(enumeration.stdout,enumeration.stderr)
        listing=client('ls');assert all(p in listing for p in ('GB','GBC','GBA'));assert 'Config' not in listing and 'secret' not in listing
        bad=root/'bad-auth';bad.write_text('username = slot\npassword = WRONG\n');bad.chmod(0o600)
        client('ls',False,authfile=bad)
        client('ls',False,protocol='SMB2')
        source=root/'upload.gba';source.write_bytes(bytes(1024))
        # The client honors server-required encryption even with its preference off.
        # Observe actual wire frames, so a plaintext file exchange cannot pass unnoticed.
        with EncryptionProbe() as probe:
            client('ls',protection='off',port=probe.port)
            client(f'put {source} GBA/Privacy.gba',protection='off',port=probe.port)
            assert probe.encrypted>0,'no encrypted SMB3 transform frames observed'
            assert not probe.plain_file_commands,('plaintext file operations',probe.plain_file_commands)
        assert (root/'Games/GBA/Privacy.gba').read_bytes()==source.read_bytes()
        client(f'put {source} GBA/Uploaded.gba')
        assert (root/'Games/GBA/Uploaded.gba').read_bytes()==source.read_bytes()
        client(f'put {source} GBA/Étoile.gba')
        assert (root/'Games/GBA/Étoile.gba').read_bytes()==source.read_bytes()
        wallpaper=root/'wallpaper.png'
        wallpaper.write_bytes((script.parents[2]/'assets/background_default.png').read_bytes())
        client(f'put {wallpaper} Uploaded.png',share='Wallpapers')
        assert (root/'Wallpapers/Uploaded.png').read_bytes()==wallpaper.read_bytes()
        (root/'Games/private-link').symlink_to(root/'Config/secret')
        client(f'get private-link {root}/leaked',False)
        client(f'get ../Config/secret {root}/leaked',False)
        client('ls',False,share='Config')
        assert not (root/'leaked').exists() or not (root/'leaked').read_bytes()
    except BaseException:
        for log in sorted((run/'slot-smb').glob('test-log.*')):
            print(f'{log.name}:\n{log.read_text(errors="replace")[-12000:]}',file=sys.stderr)
        raise
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
