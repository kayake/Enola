# Enola
A powerful google-dork search tool

## 1 Installation

### 1.1  Debian/Kali/Ubuntu
```bash
sudo apt-get install enola
```

### 1.2 Arch Linux
```bash
sudo pacman -S enola
```

### 1.3 Termux
> [!WARNING]
> There isn't `Rust` on Termux, so use `KaliNetHunter` or install the `Bin` from the Releases

### 1.4 Others
```bash
git clone https://github.com/kayake/enola 
```


> [!TIP]
> You can make all of this executing `./install.sh`

and build by **YOURSELF**
```bash
cargo build && cd target/debug/
```

> [!NOTE]
> In windows you must install from Releases

## 2 Options
### 2.1 Mode
#### 2.1.1 API Mode
You can enable API search engine's sites
```bash
enola --target foo --apimode
```

### 2.2 Settings
#### 2.2.1 Query
If you already have queries prepared, you can use them
```bash
<...> --query myquery.txt
```

#### 2.2.2 Payload
If you have a single Dork Payload you can use it, but you must use `SITE` and `STRING` placeholder

```bash
<...> --payload "intitle:STRING inurl:SITE"
```

#### 2.2.3 Payloads/Sites
If you have lists of Sites and Payloads, it's interesting to use them instead of default ones
```bash
<...> --sites mysites.txt --payloads mypayloads.txt
```
