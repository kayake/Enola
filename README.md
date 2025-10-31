# Enola

**Enola** is a powerful social-media reconnaissance tool with an advanced Google Dork generation mode for precise searching and URL discovery.

---

## Features

* Fast generation of Google Dork payloads and targeted queries
* Multiple input modes: single query, lists of queries, sites and payloads
* Proxy support for high-volume searches and rate-limit avoidance
* Verbose logging with multiple levels (info → response)
* Cross-platform installer for Linux-based systems and Windows

---

## Table of Contents

* [Installation](#installation)

  * [Linux (Debian/Ubuntu and derivatives)](#linux)
  * [Termux (no root)](#termux)
  * [Windows](#windows)
* [Quick Start](#quick-start)
* [Options](#options)

  * [Mode](#mode)

    * [Google Dork Mode](#google-dork-mode)
  * [Settings](#settings)

    * [Query](#query)
    * [Payload](#payload)
    * [Payloads / Sites](#payloads--sites)
  * [Advanced](#advanced-settings)

    * [Verbose levels](#verbose-levels)
* [Examples](#examples)
* [Security & Usage Notes](#security--usage-notes)
* [License](#license)

---

## Installation

### Linux (Debian / Ubuntu and derivatives)

Run the bundled installer with root privileges:

```bash
sudo ./install.sh
```

This will install `enola` as a global command (typically under `/usr/bin`).

### Termux (no root)

If you are on Termux without root, installing to `/usr/bin` is not possible. Consider using **Kali NetHunter (rootless)** or installing `enola` into a local bin directory in your home (e.g. `~/.local/bin`) and adding it to your `PATH`.

Reference: [https://www.kali.org/docs/nethunter/nethunter-rootless/](https://www.kali.org/docs/nethunter/nethunter-rootless/)

### Windows

Run the provided `install.bat` **as Administrator** to register `enola` as a global command.

If you prefer not to run an installer with elevated permissions, download the standalone executable from the releases page: `releases/*.exe`.

---

## Quick Start

Basic scan with a simple target:

```bash
enola --target foo
```

Run with Google Dork generation and a proxy list:

```bash
enola --target foo --google-dork-mode --proxies my-proxies.txt
```

> [!IMPORTANT]
> Always use a proxy list or otherwise throttle requests to avoid being rate-limited or blocked by Google.

---

## Options

### Mode

`--mode` selects the scanning workflow. The primary specialized mode is **Google Dork Mode**, which dynamically composes dork payloads using your queries, site lists and payload templates.

#### Google Dork Mode

When enabled, Enola creates payload dorks by combining: queries, site placeholders, and payload templates. This greatly increases precision for targeted search discovery.

Example:

```bash
enola --target foo --google-dork-mode --proxies proxies.txt
```

> [!NOTE]
> Google imposes rate limits. Use proxies or low parallelism settings to avoid blocks.

### Settings

#### Query

Provide a single query file containing one query per line:

```bash
enola --target foo --query myqueries.txt
```

#### Payload

A payload template may include `SITE` and `STRING` placeholders that will be substituted at runtime:

```bash
enola --target foo --payload "intitle:STRING inurl:SITE --google-dork-mode"
```

`SITE` will be replaced by entries from your sites list, and `STRING` by your queries or keywords.

#### Payloads / Sites

Supply lists of payload templates and/or site hosts to use instead of the built-in defaults:

```bash
enola --target foo --sites mysites.txt --payloads mypayloads.txt
```

---

## Advanced Settings

### Verbose levels

Control output granularity with `--verbose <level>`.

|     Name | Level | Description                           |
| -------: | :---: | :------------------------------------ |
|     Info |   1   | Informational messages                |
|     Warn |   2   | Warnings                              |
|    Error |   3   | Errors only                           |
|    Found |   4   | Show discovered results               |
| NotFound |   5   | Report payloads that returned nothing |
|    Debug |   6   | Debug output                          |
|  Request |   7   | Request-level details                 |
| Response |   8   | Full response details (headers/body)  |

---

## Examples

Search with a custom payload and site list:

```bash
enola --target "company-name" --sites mysites.txt --payload "inurl:SITE intitle:STRING"
```

Run using a query list and proxies:

```bash
enola --target "foo" --query queries.txt --proxies proxies.txt --google-dork-mode
```

---

## Security & Usage Notes

* **Rate limits:** Google actively rate-limits and blocks automated searches. Use a proxy pool and sensible request pacing.
* **Legal & ethical use:** Only scan targets you are authorized to test. Do **not** use Enola for illegal or abusive activities.
* **Privacy:** Be careful when logging request/response bodies; they may contain sensitive information.

---

## Contributing

Contributions are welcome. Please follow these guidelines:

1. Open an issue to discuss major changes.
2. Create a feature branch for your work.
3. Add tests and documentation for non-trivial changes.
4. Submit a pull request with a clear description of your changes.

---

## License

Enola is released under the **MIT License**.

---
