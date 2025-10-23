# Git implementation

Kilde: [https://git-scm.com/book/en/v2/Git-Internals-Plumbing-and-Porcelain|GIT]

"Git is fundamentally a content-addressable filesystem with a VCS user interface written on top of it".


## Plumbing and Porcelain

Git init lager `.git` mappen, som inneholder så og si alt du trenger

Kjernen av GIT er:
- HEAD: Peker på toppen nåværende branchen
- index: staging info
- objects: inneholder alt innholdet
- refs: pekere på commit objekter (branches, tags, remotes ++)

Andre ting i `.git`:
- config: konfigurasjon til git
- info: inneholder .gitignore greier


Content-addressable filesystem: key-value store

blob: en versjon av innholdet i en fil (filinnhold eller inodes).
tree-objects: inneholder mappestrukturen. dvs tree peker på blobs eller andre trees.

Plumbing commands
- `git hash-object`:  Tar innhold og konverterer det til en hash-nøkkel (40-character SHA-1 checksum). 
        `-w` skriver det så til til databasen
- `git cat-file`: Svært andvenlig: Viser frem innholdet i git objekter
- `git update-index`:
- `git write-tree`:
- `git commit-tree`:
The sum of these commands is `git add` and `git commit`

The commit object
- Top level tree for the snapshot of the project at that point
- The parent commits (if it exists)
- author/commiter (bassed on user.name and user.email)
- blank line
- commit message
    
### Object storage:
gitt innhold `content`:
```
blob: 
header = "blob #{sizeof(content)}\0"
store = header + content
sha1 = SHA1(store)
```
change blob to tree or commit for de andre.

commit and tree are very specific

git compresses content with zlib, which is stored in `.git/<first 2 digits>/<final 38 digits>`
    https://github.com/rust-lang/flate2-rs
    https://crates.io/crates/sha1


### References
For å finne igjen hasher bruker vi `git/refs`

```sh
echo <hash> > .git/refs/heads/master
```
Ikke anbefalt, men mulig. Vanligvis bruker man `git update-ref`

For å sjekke ut til en branch trenger man dermed bare lage en referanse til en eller annen commit
```sh
git update-ref refs/heads/test cac0ca
```
cat .git/HEAD viser hvilken branch man er på. git commit bruker SHA-1 verdien som
ligger her som forelder

read: git symbolic-ref HEAD
write: git symbolic-ref HEAD refs/heads/test

vi kan også har refs til den fjerde type object: tags
vi ser bort fra tags for nå

Remotes
git lagrer den siste hashen som ble pushet til remote i
f.eks. `.git/refs/remotes/origin/master`

disse er å anse som read-only

## Packfiles
Hvis vi gjør som beskrevet over ender vi opp med masse nesten-duplikate blobs
Derfor minimerer git innhold i packfiles

porcelain: `git gc`: pakker data
plumb: `git verify-pack`

- gjøres jevnlig
- den nyeste versjonen har alt innholdet, mens eldre veersjoner inneholder en diff

## Refspec
git remote add legger til innhold i .git/config
inneholder url og "fetch = +<src>:<dst>

## Pushe til remote
The dum protocol

`git send-pack`
`git receive-pack`









