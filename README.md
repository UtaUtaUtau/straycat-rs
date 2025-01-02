# straycat-rs ![build](https://github.com/UtaUtaUtau/straycat-rs/actions/workflows/build.yml/badge.svg)
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

An official resampler manifest file is now available for OpenUtau users [here.](https://raw.githubusercontent.com/UtaUtaUtau/straycat-rs/master/straycat-rs.yaml) Right click and select `Save as...`
# Example Renders
 These renders use straycat-rs 1.0.10. No flags are used in these renders unless stated.
 
 **Voicebank**: 電圧空 -Halcyon- / Denatsu Sora -Halcyon- / VCV

https://github.com/user-attachments/assets/13fa2f68-a735-41fe-8921-4432d064df44

 **Voicebank**: 紅 通常 / Kurenai Normal / VCV

https://github.com/user-attachments/assets/7fc60080-fe27-46a0-b1e5-ea54c84a9d7c

 **Voicebank**: 戯白メリー Highwire / Kohaku Merry Highwire / VCV

https://github.com/user-attachments/assets/3b81d548-2e51-4a96-a25a-a062fd35094e

 **Voicebank**: 水音ラル float / Mine Laru float / VCV

https://github.com/user-attachments/assets/c1c04873-4839-4adf-a6cf-fe82c4bcbe44

 **Voicebank**: 吼音ブシ-武- / Quon Bushi -武- / VCV

https://github.com/user-attachments/assets/04843cd9-51ac-49c4-82c0-dc8f4572f501

 **Voicebank**: 廻音シュウVer1.00 / Mawarine Shuu Ver1.00 / VCV

https://github.com/user-attachments/assets/10ac2ad4-e78d-4533-b6db-b86653425602

 **Voicebank**: Number Bronze・ate / CVVC

https://github.com/user-attachments/assets/bf9169ca-7fb8-494e-850a-d03eab7fd3e9

 **Voicebank**: 学人デシマル χΩ / Gakuto Deshimaru Chi-Omega / CVVC

https://github.com/user-attachments/assets/13d05c6b-296d-4a2d-9ce7-04a652cce51c

 **Voicebank**: CZloid / English VCCV / Uses P0p-1 for CCs

https://github.com/user-attachments/assets/0fb6a861-b71d-4d40-b04a-d1d351379a15

# Remarks
 This resampler will not be an exact copy of [straycat](https://github.com/UtaUtaUtau/straycat), but a variation of it. It may not do the exact same things as straycat, but my goal with this resampler is to match or surpass the quality of straycat.

 I am also not obliged to transfer the flags from straycat to straycat-rs, but if I do, I will most likely add improvements to it to give the users a better experience.

 Overall, this resampler serves to be a new and improved version of the older Python-based straycat, not a faithful translation of straycat to a compiled language.
