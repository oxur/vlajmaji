import json, pathlib, re, subprocess
p = pathlib.Path('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/planning')
a = p/'project01-multi-source-dictionary/arc02-reproducible-jbotci-import'
project = a.parent
final = '--final' in __import__('sys').argv
results = {}
def ledger(path,prefix,expected,done):
    rows=[line for line in path.read_text().splitlines() if re.match(r'^\| '+re.escape(prefix)+r'\d+ \|',line)]
    assert len(rows)==expected,(str(path),len(rows),expected)
    statuses=[line.split('|')[6].strip() for line in rows]
    assert sum(x.startswith('done') for x in statuses)==done,(str(path),statuses,done)
    results[str(path.relative_to(p))]={'rows':len(rows),'done':done,'open':sum(x=='open' for x in statuses)}
ledger(a/'ledger.md','A02-',8,8 if final else 2)
ledger(a/'slice01-lossless-export-normalization/ledger.md','A02S01-',7,7)
ledger(a/'slice02-pinned-pipeline-and-persisted-lookup/ledger.md','A02S02-',8,8)
ledger(project/'ledger.md','P01-',14,0)
assert not list(project.glob('arc03-*'))
files=[p/'AGENTS.md',project/'project-plan.md',project/'ledger.md',project/'design.md']+[a/x for x in ('arc-plan.md','ledger.md','closing-report.md','cdc-verification.md','cdc-retrospective.md')]
links=0
for file in files:
    assert file.is_file(),str(file)
    text=file.read_text()
    for target in re.findall(r'\]\(([^)]+)\)',text):
        if target.startswith(('http:','https:','#','mailto:')): continue
        local=target.strip('<>').split('#')[0]
        if not local: continue
        assert (file.parent/local).exists(),(str(file),target)
        links+=1
results['canonical_files']=len(files)
results['local_links']=links
for location,commit in [('/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc02/vlajmaji','4e3701606be7d9a24a863468ea4fdbd7c1b369ab'),('/Users/oubiwann/lab/oxur/vlajmaji','d1e1c928f7999f2220b046cc5a5817f0529312ca'),('/Users/oubiwann/lab/oxur/bauske','c5801ae17caaf86d2651b5ce34570199e873ad0f')]:
    head=subprocess.check_output(['git','-C',location,'rev-parse','HEAD'],text=True).strip()
    status=subprocess.check_output(['git','-C',location,'status','--porcelain'],text=True)
    assert head==commit,(location,head)
    assert not status,(location,status)
    results[location]={'head':head,'clean':True}
print(json.dumps(results,indent=2))
