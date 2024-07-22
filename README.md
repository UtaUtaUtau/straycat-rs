# straycat-rs
 A Rust port of straycat, a WORLD-based UTAU resampler

# How to use
 Download the [latest version](https://github.com/UtaUtaUtau/straycat-rs/releases/latest/download/straycat-rs.exe) of straycat-rs and use it like a regular UTAU resampler.
# How to compile
 **Note**: By the nature of an UTAU resampler, it is only ideal to build this program in Windows.
 1. Install [rustup](https://rustup.rs/).
 2. Decide whether you want to build with the icon.
    - Build with icon:
        1. Install [Windows SDK](https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/).
        2. Locate `rc.exe`. It is usually in `C:\Program Files (x86)\Windows Kits\10\bin\<version number>\x64\rc.exe`
        3. Replace the location for `rc.exe` in the build script `build.rs`.
        4. Build with `cargo build -r`
    - Build without icon:
        1. Delete the build script `build.rs`.
        2. Build with `cargo build -r`
# Flag Documentation
Check flag documentation [here](flag_docs.md).
# Example Renders
 These renders use straycat-rs 1.0.1. No flags are used in these renders unless stated.
 
 **Voicebank**: 電圧空 -Halcyon- / Denatsu Sora -Halcyon- / VCV
 
https://github.com/user-attachments/assets/2c214631-fb20-4d96-a2b2-8000bd204dc9

 **Voicebank**: 紅 通常 / Kurenai Normal / VCV

https://github.com/user-attachments/assets/cdea650e-8291-4632-8e3b-6e03c888db85

 **Voicebank**: 戯白メリー Highwire / Kohaku Merry Highwire / VCV
 
https://github.com/user-attachments/assets/fb39470e-a0a2-42ba-8c4e-85e6847e559c

 **Voicebank**: 水音ラル parse2 / Mine Laru parse2 / VCV

https://github.com/user-attachments/assets/269db252-aad1-4501-a8b9-485d1de445f6

 **Voicebank**: 吼音ブシ-武- / Quon Bushi -武- / VCV

https://github.com/user-attachments/assets/76cbf0d2-4edc-4118-ad92-2c5415db1cce

 **Voicebank**: 廻音シュウVer1.00 / Mawarine Shuu Ver1.00 / VCV

https://github.com/user-attachments/assets/fc057da4-8f0c-4d60-95a0-31ee8b8ca9ae

 **Voicebank**: Number Bronze・ate / CVVC
 
https://github.com/user-attachments/assets/036d10ba-81cf-4052-ad29-78d39fc4c08f

 **Voicebank**: 学人デシマル χΩ / Gakuto Deshimaru Chi-Omega / CVVC

https://github.com/user-attachments/assets/e07aacb1-175f-43fc-b4b4-a92711c7b9d7

 **Voicebank**: CZloid / English VCCV / Uses P0p-1 for CCs

https://github.com/user-attachments/assets/889d27fd-6e64-4e14-971f-2bb614f46b6c

# Remarks
 This resampler will not be an exact copy of [straycat](https://github.com/UtaUtaUtau/straycat), but a variation of it. It may not do the exact same things as straycat, but my goal with this resampler is to match or surpass the quality of straycat.

 I am also not obliged to transfer the flags from straycat to straycat-rs, but if I do, I will most likely add improvements to it to give the users a better experience.

 Overall, this resampler serves to be a new and improved version of the older Python-based straycat, not a faithful translation of straycat to a compiled language.
