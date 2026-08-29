# lmv - переміщує файл/директорію на нове місце, залишаючи симлінк на старому

---

# Встановлення:
```bash
git clone https://github.com/Undercat037/lmv.git
cd lmv
cargo build --release && sudo cp -r target/release/lmv /usr/bin/lmv
lmv --gen-completions bash | sudo tee /usr/share/bash-completion/completions/lmv
lmv --gen-completions zsh | sudo tee /usr/share/zsh/site-functions/_lmv
lmv --gen-completions fish | sudo tee /usr/share/fish/vendor_completions.d/lmv.fish
```

<details>
<summary><h3>Arch(AUR)</h3></summary>

### AUR
```bash
paru -S lmv
```
### Makepkg
```bash
git clone https://aur.archlinux.org/lmv.git
cd lmv
makepkg -si
```
</details>

# Використання:
```bash
lmv [ПРАПОРИ] <джерело> <призначення>
```

<details>
<summary><h3>Demo:</h3></summary>

```bash

╭──(deltacat㉿max-tac)─(2026/08/29)(14:29:06)-[~/.tmp/lmv-demo]
╰─❯❯ ls
drwxr-xr-x  - deltacat 29 сер 14:26  testdir1
drwxr-xr-x  - deltacat 29 сер 14:26  testdir2
.rw-r--r-- 18 deltacat 29 сер 14:28  testfile1.txt

╭──(deltacat㉿max-tac)─(2026/08/29)(14:29:12)-[~/.tmp/lmv-demo]
╰─❯❯ lmv testfile1.txt testfile1-1.txt 

╭──(deltacat㉿max-tac)─(2026/08/29)(14:29:37)-[~/.tmp/lmv-demo]
╰─❯❯ ls
drwxr-xr-x  - deltacat 29 сер 14:26  testdir1
drwxr-xr-x  - deltacat 29 сер 14:26  testdir2
.rw-r--r-- 18 deltacat 29 сер 14:28  testfile1-1.txt
lrwxrwxrwx  - deltacat 29 сер 14:29  testfile1.txt -> /home/deltacat/.tmp/lmv-demo/testfile1-1.txt

╭──(deltacat㉿max-tac)─(2026/08/29)(14:29:39)-[~/.tmp/lmv-demo]
╰─❯❯ lmv -rn testfile1-1.txt testdir1/
would move 'testfile1-1.txt' -> 'testdir1/testfile1-1.txt' and leave relative symlink

╭──(deltacat㉿max-tac)─(2026/08/29)(14:30:35)-[~/.tmp/lmv-demo]
╰─❯❯ ls
drwxr-xr-x  - deltacat 29 сер 14:26  testdir1
drwxr-xr-x  - deltacat 29 сер 14:26  testdir2
.rw-r--r-- 18 deltacat 29 сер 14:28  testfile1-1.txt
lrwxrwxrwx  - deltacat 29 сер 14:29  testfile1.txt -> /home/deltacat/.tmp/lmv-demo/testfile1-1.txt

╭──(deltacat㉿max-tac)─(2026/08/29)(14:30:41)-[~/.tmp/lmv-demo]
╰─❯❯ lmv -r testfile1-1.txt testdir1/

╭──(deltacat㉿max-tac)─(2026/08/29)(14:31:09)-[~/.tmp/lmv-demo]
╰─❯❯ ls
drwxr-xr-x - deltacat 29 сер 14:31  testdir1
drwxr-xr-x - deltacat 29 сер 14:26  testdir2
lrwxrwxrwx - deltacat 29 сер 14:31  testfile1-1.txt -> testdir1/testfile1-1.txt
lrwxrwxrwx - deltacat 29 сер 14:29  testfile1.txt -> /home/deltacat/.tmp/lmv-demo/testfile1-1.txt

╭──(deltacat㉿max-tac)─(2026/08/29)(14:31:12)-[~/.tmp/lmv-demo]
╰─❯❯ lmv -r testdir1 testdir2/123dir123

╭──(deltacat㉿max-tac)─(2026/08/29)(14:31:43)-[~/.tmp/lmv-demo]
╰─❯❯ ls testdir2/
drwxr-xr-x - deltacat 29 сер 14:31  123dir123

╭──(deltacat㉿max-tac)─(2026/08/29)(14:31:55)-[~/.tmp/lmv-demo]
╰─❯❯ ls
lrwxrwxrwx - deltacat 29 сер 14:31  testdir1 -> testdir2/123dir123
drwxr-xr-x - deltacat 29 сер 14:31  testdir2
lrwxrwxrwx - deltacat 29 сер 14:31  testfile1-1.txt -> testdir1/testfile1-1.txt
lrwxrwxrwx - deltacat 29 сер 14:29  testfile1.txt -> /home/deltacat/.tmp/lmv-demo/testfile1-1.txt

╭──(deltacat㉿max-tac)─(2026/08/29)(14:31:57)-[~/.tmp/lmv-demo]
╰─❯❯ lmv -ra  testdir2/123dir123/testfile1-1.txt ./testfile.

╭──(deltacat㉿max-tac)─(2026/08/29)(14:33:17)-[~/.tmp/lmv-demo]
╰─❯❯ ls
lrwxrwxrwx  - deltacat 29 сер 14:31  testdir1 -> testdir2/123dir123
drwxr-xr-x  - deltacat 29 сер 14:31  testdir2
.rw-r--r-- 18 deltacat 29 сер 14:28  testfile.
lrwxrwxrwx  - deltacat 29 сер 14:31  testfile1-1.txt -> testdir1/testfile1-1.txt
lrwxrwxrwx  - deltacat 29 сер 14:29  testfile1.txt -> /home/deltacat/.tmp/lmv-demo/testfile1-1.txt

╭──(deltacat㉿max-tac)─(2026/08/29)(14:33:18)-[~/.tmp/lmv-demo]
╰─❯❯ 
```
</details>

# Прапори:
| Прапор | Опис |
|---|---|
| `-v`, `--verbose` | Докладний вивід |
| `-n`, `--dry-run` | Показати, що буде зроблено, нічого не змінюючи |
| `-f`, `--force` | Перезаписати призначення, якщо воно вже існує |
| `-a`, `--archive` | Зберігати права доступу та час модифікації |
| `-P`, `--progress` | Показувати прогрес копіювання (кількість файлів) |
| `-r`, `--relative` | Відносний симлінк (`../../...`) замість абсолютного |
| `--gen-completions <SHELL>` | Згенерувати автодоповнення (`bash`/`zsh`/`fish`) |
| `-h`, `--help` | Довідка |
| `-V`, `--version` | Версія |