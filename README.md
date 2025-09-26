# Beacon LN

**Your guiding light on the Lightning Network.**

---

<svg width="400" height="100" xmlns="http://www.w3.org/2000/svg">
    <style>
        .title { font: bold 60px sans-serif; fill: #333; }
        .ln { font: bold 60px sans-serif; fill: #F7931A; } /* Bitcoin Orange */
        .beam { fill: #F7931A; opacity: 0.2; }
    </style>
    <polygon class="beam" points="5,5 45,50 5,95" />
    <text x="60" y="75" class="title">Beacon</text>
    <text x="310" y="75" class="ln">LN</text>
</svg>

An open-source desktop Lightning Node focused on simplicity and user experience, powered by LDK and Rust.

## The Vision

Running a personal Lightning node is a powerful way to achieve self-sovereignty and support the Bitcoin network. However, the process is often intimidating for non-technical users. **Beacon LN** aims to solve this by providing an intuitive, secure, and easy-to-use desktop application for managing your Lightning node.

This project is built with Rust, [Lightning Development Kit (LDK)](https://lightningdevkit.org/), and the [iced](https://github.com/iced-rs/iced) GUI framework.

## Core Goals for v1.0

This list outlines the planned features for the first stable release of Beacon LN.

* [ ] **Full Node Control:** Open/close channels, send/receive payments, and manage your liquidity.
* [ ] **Intuitive Dashboard:** A clean user interface to visualize your channels, balance, and routing activity.
* [ ] **Simple Setup:** No command-line or complex configuration files required.
* [ ] **Cross-Platform:** Works on Windows, macOS, and Linux.
* [ ] **Self-Custodial:** You always control your own keys.

## How to Contribute

This is an open-source project and we welcome contributions! Whether it's reporting a bug, suggesting a feature, or writing code, your help is appreciated. Please see our `CONTRIBUTING.md` file for guidelines.

## Get Support

If you have a question or run into an issue, please [open an issue](https://github.com/Anyitechs/beacon-ln/issues) on our GitHub repository.