# Enola
A powerful Google-dork search tool

## 1 Installation

### 1.1 Debian / Kali / Ubuntu
```bash
sudo apt-get install enola
```
### 1.2 Arch Linux

```bash
sudo pacman -S enola
```
### 1.3 Termux

> [!WARNING]
> `Rust` is not officially supported on `Termux`. Use `Kali NetHunter` or download the binary from the Releases.

### 1.4 Others

```bash
git clone https://github.com/kayake/enola
```

> [!TIP]
> You can do all of this by running `./install.sh`.

Or build it yourself:

```bash
cargo build && cd target/debug/
```

> [!NOTE]
> On Windows, you must install from the Releases.


## 2 Options

### 2.1 Mode

### 2.1.1 API Mode

Enable API search engine sites:

```bash
enola --target foo --apimode
```

### 2.2 Settings

#### 2.2.1 Query

If you already have queries prepared, use them:

```bash
enola --query myquery.txt
```

#### 2.2.2 Payload

Use a single dork payload with `SITE` and `STRING` placeholders:

```bash
enola --payload "intitle:STRING inurl:SITE"
```

#### 2.2.3 Payloads / Sites

Use lists of sites and payloads instead of the default ones:

```bash
enola --sites mysites.txt --payloads mypayloads.txt
```

### 2.3 Advanced Settings

#### 2.3.1 Verbose

##### Levels

Name      | Level | Description
--------- | ----- | ----------------------------
Info      | 1     | Displays info messages
Warn      | 2     | Displays warnings
Error     | 3     | Displays errors
Found     | 4     | Shows found results
NotFound  | 5     | Shows payloads not found
Debug     | 6     | Displays debug messages
Request   | 7     | Shows request details
Response  | 8     | Shows response details
