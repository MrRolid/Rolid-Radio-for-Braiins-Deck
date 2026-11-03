# License scope

Original RoliD Radio source code is MIT (LICENSE). This does not relicense third-party code, fonts, dependencies or built artifacts.

The WASM widget statically links Braiins bmc-wasm-sdk and associated components under GPL-3.0-or-later. The combined WASM/NAR is distributed under GPL-3.0-or-later, with MIT permissions retained for original RoliD source. GPL text: LICENSES/GPL-3.0.txt. The launcher is copied from Braiins under GPL. Braiins source: https://github.com/BraiinsForge/bmc-main at b2c945b5f7a8ea3a7d38a8410a1ca64e8b69b9af. The accompanying source-dependencies.tar.gz supplies the pinned SDK and Cargo dependency sources with original notices. Distributors must provide corresponding source for the combined widget, including that SDK revision and dependency source; a source link alone is not a replacement for those obligations.

Japanese vector labels derive from Noto Sans JP: assets/jp/OFL.txt contains the SIL OFL and copyright notices. Font assets retain that license. Cargo dependency versions are recorded in each Cargo.lock; retain dependency license notices when redistributing builds. FFmpeg, madplay and ALSA utilities are external installed programs and are not included in this release.

Do not describe the combined widget binary as MIT-only. See https://www.gnu.org/licenses/gpl-faq.en.html#IfLibraryIsGPL.
