# Enola
A powerful Google-dork and site search tool

## 1 Installation

### Based Linux

```bash
sudo install.sh
```

> [!IMPORTANT]
> `sudo` is required. If you are using `Termux` without `root`, consider installing `Kali NetHunter (Rootless)`.

### Windows

You will exec `install.bat` with admin permissions
> [!NOTE]
> You may ask, why super admin? Because the code will make `enola` as a global command. If uou don't agree, you download from [`releases/.exe`](https://github.com/kayake/enola/tags/releases/)
## 2 Options

### 2.1 Mode

### 2.1.1 Google Dork Mode

Enabling this future, it will create payload dorks for more accurate search

> [!WARNING]
> You must use some proxy List due to Google rate limit.

```bash
enola --target foo --google-dork-mode --proxies YOUR_LIST_PROXY.txt
```

> [!IMPORTANT]
> If there is a custom Query, Payload, Sites (to put in dork), etc. You should use them by adding `--queries`, `--sites`, `--paylods`. For more information, use `--help`.

### 2.2 Settings

#### 2.2.1 Query

If you already have queries prepared, use them:

```bash
enola --target foo --query myquery.txt
```

#### 2.2.2 Payload

Use a single dork payload with `SITE` and `STRING` placeholders:

```bash
enola --target foo --payload "intitle:STRING inurl:SITE --google-dork-mode"
```

#### 2.2.3 Payloads / Sites

Use lists of sites and payloads instead of the default ones:

```bash
enola --target foo --sites mysites.txt --payloads mypayloads.txt
```

#### 2.3.4 API Sites

Use lists of third sites' search engine:

```bash
enola --target foo --api-sites
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
