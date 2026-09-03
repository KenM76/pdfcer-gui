import io

p = 'crates/pdfcer-gui/src/app/actions/apply.rs'
lines = io.open(p, encoding='utf-8').read().split('\n')

start = next(i for i, l in enumerate(lines) if 'A fit sets the scale AND asks for the view' in l)
end = next(i for i in range(start, len(lines)) if 'The seven view verbs' in l or 'The seven view verbs' in lines[i])
orphan = lines[start:end]
del lines[start:end]
io.open(p, 'w', encoding='utf-8').write('\n'.join(lines))

# put it where the code went
p = 'crates/pdfcer-gui/src/app/actions/view.rs'
s = io.open(p, encoding='utf-8').read()
block = '\n'.join('        ' + l.strip() for l in orphan if l.strip())
s = s.replace("        Action::Fit(mode) => {", block + "\n        Action::Fit(mode) => {", 1)
io.open(p, 'w', encoding='utf-8').write(s)
print('moved')
