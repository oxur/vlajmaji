from pathlib import Path
import hashlib,json,subprocess,sys
r=Path(__file__).parent
binary=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target/debug/examples/import_jbotci')
pin=Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc02/data/jbotci/3d9fd41d9ed32de26311ee27ec86b6b165985aa1')
canonical=(r/'s01-candidate03-full-api.json').read_bytes()
example=r/'s01 example candidate03.json';cli=r/'s01 cli candidate03.json'
assert example.read_bytes()==canonical and cli.read_bytes()==canonical+b'\n'
expected=hashlib.sha256(canonical).hexdigest();cases=[]
for name,args,output,error in [
 ('occupied',[str(pin/'dictionary-en.json'),str(pin/'dictionary-en.metadata.toml'),str(example)],example,b'AlreadyExists'),
 ('invalid-pin',[str(r/'api-fixtures03/duplicate-native-and-complete/raw.json'),str(pin/'dictionary-en.metadata.toml'),str(r/'example-invalid-pin03.json')],r/'example-invalid-pin03.json',b'HashMismatch')]:
 before=output.read_bytes() if output.exists() else None
 cmd=[str(binary),*args];p=subprocess.run(cmd,capture_output=True)
 (r/f'example-{name}03.stdout').write_bytes(p.stdout);(r/f'example-{name}03.stderr').write_bytes(p.stderr)
 assert p.returncode!=0 and p.stdout==b'' and error in p.stderr,(name,p.returncode,p.stdout,p.stderr)
 assert (output.read_bytes() if output.exists() else None)==before
 cases.append({'name':name,'argv':cmd,'exit':p.returncode,'expected_stderr_category':error.decode(),'existing_bytes_preserved':before is not None,'new_output_absent':before is None and not output.exists()})
print(json.dumps({'verdict':'PASS','positive':'full shipped example run/reload previously exit0','negative_controls':cases,'canonical_bytes':len(canonical),'canonical_sha256':expected,'example_exact_canonical':True,'cli_exact_canonical_plus_one_lf':True,'cli_bytes':cli.stat().st_size,'cli_sha256':hashlib.sha256(cli.read_bytes()).hexdigest(),'example_binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest()},indent=2))
