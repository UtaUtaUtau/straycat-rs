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
        2. Locate `rc.exe`. It is usually in `C:\Program Files (x86)\Windows Kits\10\bin\<version number>\x86\rc.exe`
        3. Replace the location for `rc.exe` in the build script `build.rs`.
        4. Build with `cargo build -r`
    - Build without icon:
        1. Delete the build script `build.rs`.
        2. Build with `cargo build -r`
# Remarks
 This resampler will not be an exact copy of [straycat](https://github.com/UtaUtaUtau/straycat), but a variation of it. It may not do the exact same things as straycat, but my goal with this resampler is to match or surpass the quality of straycat.

 I am also not obliged to transfer the flags from straycat to straycat-rs, but if I do, I will most likely add improvements to it to give the users a better experience.

 Overall, this resampler serves to be a new and improved version of the older Python-based straycat, not a faithful translation of straycat to a compiled language.