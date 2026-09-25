# Steam Account Switcher

Unlimited Steam accounts, switched in one click, each with a rank badge.

The application lists the accounts your Steam client already remembers, switches
the account Steam logs into on its next start (restarting the client for you),
and keeps a local playtime history per account, game, day and session.

- **Rank badge per account**: Marvel Rivals, Overwatch, Counter-Strike 2 (the
  eighteen competitive ranks) or Counter-Strike 2 Premier — a rating from 0 to
  50 000, coloured like the game.
- **Cards ordered by last login**, grouped in folders, with a search field,
  per-account colours and a right-click menu. One card shows the details, a
  double click switches.
- **Linux and Windows**: AppImage and RPM on Linux, portable `.exe` or installer
  on Windows. Tray icon, start with the system, and updates from the app itself.

Steam asks for passwords and Steam Guard codes itself: the application never
reads or stores them, and everything it saves stays on your machine.

## Download

Builds are on the [releases page](https://github.com/SunkwiBOT/steam-account-switcher/releases).

## Build

```bash
npm ci
npm run dev:app     # application, interface hot reloaded
npm run build:app   # release executable with the interface embedded
npm run check       # types, lint, formatting and Rust lints
npm run test:rust   # unit tests
```

Rust and Node 22.12 or newer, plus the
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your system.

## License

MIT — see [LICENSE](LICENSE).

## Disclaimer

This is an independent, unofficial tool. It is not affiliated with, endorsed by
or sponsored by Valve Corporation, Blizzard Entertainment, Marvel, NetEase or
any other game publisher.

Steam, Counter-Strike 2, Overwatch, Marvel Rivals and their names, logos and
rank artwork are trademarks or registered trademarks of their respective
owners. The rank badges bundled in `ui/public/ranks/` are used only to identify
the games they belong to, and all rights to them stay with their owners.

Use it at your own risk. Steam handles its own sign-in: this application never
reads or stores your password or Steam Guard codes.
