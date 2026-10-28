"""Build the SAME54-request dependency graph for original four-literal Tiny IDs.
Uses caller's pre-input typed keys + exact sealed input bytes, never query results
as the lexical/native oracle. SOURCE preparation, no child/product/store IO.
"""
import copy,hashlib,json,stat,sys
from pathlib import Path
HERE=Path(__file__).parent
PACKS=['lensisku-jbotci','llg','la-lojban','sutysisku']

def run(context_file,outfile):
 ctxfile=Path(context_file);out=Path(outfile)
 assert ctxfile.is_absolute() and ctxfile.is_relative_to(HERE) and '..' not in ctxfile.parts
 ctx=json.loads(ctxfile.read_bytes());assert ctx['fixture_scope']=='original-four-pack-synthetic-readable'
 docs={}
 for inp in ctx['inputs']:
  p=Path(inp['release']);assert p.is_absolute() and p.is_relative_to(ctxfile.parent)
  walk=ctxfile.parent;assert stat.S_ISDIR(walk.lstat().st_mode) and not stat.S_ISLNK(walk.lstat().st_mode)
  for atom in p.relative_to(ctxfile.parent).parts:
   walk/=atom;assert not stat.S_ISLNK(walk.lstat().st_mode)
  assert stat.S_ISREG(p.lstat().st_mode)
  raw=p.read_bytes();assert hashlib.sha256(raw).hexdigest()==inp['release_sha256']
  docs[inp['pack_id']]=json.loads(raw)
 assert set(docs)==set(PACKS)
 def witness(pack):
  d=docs[pack];assertions=[a for a in d['metadata']['assertions'] if a['eligibility']=='Lexical']
  a=assertions[0];k=ctx['expected_keys'][a['id']]
  release=next(r for r in d['metadata']['releases'] if a['id'] in r['assertions'])
  return a,{'Catalog':{'source':k['source'],'release':release['id'],'native_entry_id':k['native_entry_id'],'native_assertion_id':k['native_assertion_id'],'occurrence':k['occurrence']}}
 jb,jbc=witness('lensisku-jbotci');la,lac=witness('la-lojban');llg,llgc=witness('llg');sut,sutc=witness('sutysisku')
 assert jbc['Catalog']['native_assertion_id']==lac['Catalog']['native_assertion_id']=='333'
 assert jbc!=lac and llgc['Catalog']['native_entry_id']=='farvi' and sutc['Catalog']['native_entry_id']=='farvi'
 missing=copy.deepcopy(jbc);missing['Catalog']['native_assertion_id']='missing-native-assertion'
 targets={'jbotci-local':{'Local':{'Assertion':jb['id']}},'jbotci-catalogue-native333':jbc,'la-catalogue-native333':lac,'llg-component':llgc,'sut-native-entry':sutc,'wrong-role':{'Local':{'Entry':jb['id']}},'missing':missing}
 rows=json.loads((HERE/'numeric-caller01/fixtures/requests-full03.json').read_bytes())
 for row in rows:
  identity=row['id'].removeprefix('prime--')
  if identity in targets:row['target']=targets[identity]
 assert len(rows)==54 and sum(bool(x.get('prime')) for x in rows)==27
 assert out.is_absolute() and out.is_relative_to(HERE) and '..' not in out.parts
 with out.open('x') as f:json.dump(rows,f,indent=2,ensure_ascii=False);f.write('\n')
 return {'requests':54,'priming':27,'measured':27,'scope':'only five real reference witnesses +wrong-role/missing IDs rebound from exact pre-input original four-literal Tiny; request IDs/options/cursor graph/error remain original'}
if __name__=='__main__':
 assert len(sys.argv)==3
 print(json.dumps(run(sys.argv[1],sys.argv[2])))
