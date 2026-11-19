# Security policy

## Supported versions

Only the latest release gets security fixes. The app offers new versions in
**Settings › Updates**; please check that the problem still happens on the
latest one before reporting.

## Reporting a vulnerability

Please do not report a vulnerability in a public issue, discussion or pull request.

Report it privately with GitHub's private vulnerability reporting: open the repository's
**Security** tab and click **Report a vulnerability**. Only you and the maintainers see
the report, and it stays private until its advisory is published.

## What to include

- The version (**Settings › Updates**, or the first line of the logs from
  **Settings › Logs**) and the operating system (Windows or Linux, and distribution).
- The part concerned, for example account switching, Steam Guard, the vault, the
  backup/import flow or the updater.
- The steps to reproduce it, or a proof of concept, and what an attacker gains.
- Logs when they help: **Settings › Logs › Open folder** lets you attach the log file.

Never send a real Steam password, a `.maFile`/`.steamguard` file, a vault backup, the
vault password, or the app's database file.

## After a report

The report is discussed in its private advisory. Once a fix is released, the advisory
is published, with credit to you if you wish.

## Scope

This repository's code: the desktop app (account switching, Steam Guard, the vault and
backups, the tray, settings) and the release and update process. Problems in Steam
itself, in the `steamguard` crate it depends on, or in GitHub's infrastructure belong to
those projects.
