# Shiryoku

> 視力  
> Shiryoku  
> Vision

Shiryoku is a terminal-based utility designed to explore the mechanics of email protocols, asynchronous scheduling, and telemetry. It provides a keyboard-driven interface for composing emails using Markdown, managing attachments, and scheduling delivery across different time zones.

The project consists of a Rust-based TUI (Text User Interface) client and a serverless backend responsible for handling scheduling logic and telemetry aggregation.

## Objectives

This project was developed for educational and instructional purposes. It serves as a case study in:
*   Building complex terminal user interfaces in Rust.
*   Implementing SMTP (Simple Mail Transfer Protocol) clients manually over raw TCP sockets.
*   Handling multipart MIME data and file attachments.
*   Designing distributed systems where state is shared between a local client and a remote edge worker.

## Features

**Composition and Interface**
*   **Markdown Support:** Compose emails using Markdown syntax, which is compiled to HTML before delivery.
*   **External Editor Integration:** Seamlesly integrates with system editors ($EDITOR, Vim, Nano) for drafting body content.
*   **Keyboard-Driven Workflow:** Optimized for efficiency with Vim-like navigation and shortcuts.

**Delivery and Scheduling**
*   **SMTP Dispatch:** Direct support for SMTP relaying via Cloudflare Workers, handling implicit SSL and authentication handshake manually.
*   **Server-Side Scheduling:** Offloads email scheduling to a remote worker, allowing the client to go offline while ensuring delivery occurs at the precise target time.
*   **Time Zone Intelligence:** Handles complex time zone conversions, ensuring emails arrive relative to the recipient's local time.

**Telemetry**
*   **Read Tracking:** Embeds invisible pixel trackers to detect when an email is opened.
*   **Metadata Aggregation:** Captures non-identifiable metadata such as geolocation (Country/City) and User-Agent strings to visualize engagement.
*   **Dashboard:** A built-in terminal dashboard to visualize open rates and logs.

## Privacy and Ethical Use Disclaimer

**Strictly for Educational Use.**

This software includes features that allow for the tracking of email interaction (pixel tracking). While this technology is standard in the marketing industry, its use by individuals carries significant privacy implications.

*   **Consent:** Do not track individuals without their explicit consent.
*   **Privacy Laws:** Users are responsible for adhering to all applicable privacy laws (such as GDPR in Europe or CCPA in California). Tracking IP addresses and location data may be illegal in certain jurisdictions without proper disclosure.
*   **No Malicious Use:** This tool must not be used for harassment, spam, or surveillance.

The authors and contributors of Shiryoku assume no liability for the misuse of this software. By using this tool, you agree to use it responsibly and ethically.

## Architecture

Shiryoku operates on a split architecture:
1.  **The Client:** A Rust binary running locally on the user's machine.
2.  **The Backend:** A Cloudflare Worker utilizing D1 (SQL database) to store logs, attachments, and pending schedules.

## Installation

### Client (Rust)

Ensure you have the Rust toolchain installed (Cargo and rustc).

1.  Clone the repository.
2.  Build the release binary:
    ```bash
    cargo build --release
    ```
3.  The binary will be located at `./target/release/shiryoku`.

### Backend (Node/Cloudflare)

The backend is built to run on the Cloudflare Workers runtime. You will need `npm` (Node Package Manager) and `wrangler` installed.

1.  Navigate to the backend directory:
    ```bash
    cd shiryoku-backend
    ```

2.  Install dependencies:
    ```bash
    npm install
    ```

3.  Authenticate with Cloudflare:
    ```bash
    npx wrangler login
    ```

4.  Create the D1 Database:
    ```bash
    npx wrangler d1 create shiryoku-db
    ```
    *Note: Copy the `database_id` output by this command and update `wrangler.toml` accordingly.*

5.  Apply the database schema:
    ```bash
    npx wrangler d1 execute shiryoku-db --file=schema.sql
    ```

6.  Deploy the worker:
    ```bash
    npx wrangler deploy
    ```

7.  **Configuration:**
    Once deployed, you must configure the API Secret.
    ```bash
    npx wrangler secret put API_SECRET
    ```

## Configuration

Upon first launch, Shiryoku will navigate to the configuration screen. You will need to provide:

*   **Identity:** Name and details for the email footer/signature.
*   **SMTP Credentials:** The username and App Password for your email provider (e.g., Gmail). These are sent securely to the backend only when scheduling an email.
*   **Worker URL:** The URL provided by Cloudflare after deploying the backend (e.g., `https://your-worker.subdomain.workers.dev`).
*   **API Secret:** The secret key you defined during backend deployment.

## License

This project is open-source. Please refer to the LICENSE file for details.
