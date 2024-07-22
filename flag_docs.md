# straycat-rs flags
 This is the main documentation of the flags available in straycat-rs.

## Value Notation for ranges
 The notation for the ranges of values follows [integer interval notation](https://en.wikipedia.org/wiki/Interval_(mathematics)#Notations_for_intervals) from mathematics. Keep in mind that flags can only receive integers (whole numbers). Here is a summary of this notation:
 - Square brackets **[]** means the value is included in the range.
 - Parentheses **()** means the value is not included in the range.
 - **inf** means infinity. The sign indicates its direction.

 Example: [0, 100) means a range from 0 to 100, including 0 but not including 100.

## Descriptors
 The behavior of the flags will be explained through a table with descriptors. The columns for these tables mean the following:
 | Column | Explanation |
 | :----: | :---------- |
 |  Flag  | The flag associated with the behavior. |
 | Description | A description of the behavior of the flag. |
 | Unit | The unit of the value related to the flag. |
 | Default | The default value of the flag. |
 | Value Range | The range of accepted values for the flag. |
 | Recommended Range | The recommended range of values for each flag. Helpful for OpenUtau users.|

# Flags
### Vocal fry flag set
 This set of flags allow producing fake vocal fries or glottal stops by adding a dip in the pitchbend that you cannot normally reach due to limitations of how resamplers receive pitchbends
 .
 | Flag | Description | Unit | Default | Value Range | Recommended Range |
 | :--: | :---------- | :--: | :-----: | :---: | :------------------------: |
 | `fe` | Enables this behavior and sets the length of the fry area. Positive values put the fry area to the left of the pivot, negative values to the right. | milliseconds | 0 | (-inf, +inf) | [-1000, 1000] |
 | `fo` | Moves the pivot, which is centered around the consonant point of the oto. Positive values move the pivot to the right, negative values to the left. | milliseconds | 0 | (-inf, +inf) | [-1000, 1000] |
 | `fl` | Changes the length of the transition to the fry. Lower values mean a faster transition. | milliseconds | 75 | [1, +inf) | [1, 250] |
 | `fv` | The volume of the fry area. | percentage | 10 | [0, 100] | [0, 100] |
 | `fp` | The pitch of the fry. | Hertz | 71 | [0, +inf) | [0, 71] |

### Devoicing flag set
This set of flags allow setting a specific area into a whispery sound, allowing the creation of fake end breaths.
 | Flag | Description | Unit | Default | Value Range | Recommended  Range |
 | :--: | :---------- | :--: | :-----: | :---: | :------------------------: |
 | `ve` | Enables this behavior and sets the length of the unvoiced area. Positive values put the unvoiced area to the left of the pivot, negative values to the right. | milliseconds | 0 | (-inf, +inf) | [-1000, 1000] |
 | `vo` | Moves the pivot, which is centered around the consonant point of the oto. Positive values move the pivot to the right, negative values to the left. | milliseconds | 0 | (-inf, +inf) | [-1000, 1000] |
 | `vl` | Changes the length of the transition. Lower values mean a faster transition. | milliseconds | 75 | [1, +inf) | [1, 1000] |

### Other flags
 These are other flags in straycat-rs that work individually.

| Flag | Description | Unit | Default | Value Range | Recommended Range |
| :--: | :---------- | :--: | :-----: | :---: | :------------------------: |
| `g`  | Shifts the formants of the render, commonly known as "gender." Higher values makes a more "masculine" quality, lower values makes a more "feminine" quality. | 10 units = 1 semitone | 0 | (-inf, +inf) | [-120, 120][^a] |
| `B`  | Controls the breathiness of the render. 100 produces a whisper only render. | percentage | 50 | [0, 100] | [0, 100][^b] |
| `P`  | Compresses the render based on the peak. Lower values give a stronger compression. 0 disables this. | percentage | 86 | [0, 100) | [0, 99] |
| `p`  | Normalizes the render based on the peak after compression. Higher values leads to a quieter normalization as the input is negated. Negative values disable this. | dB (negated) | 4 | (-inf, +inf) | [-1, 6] |
| `t`  | Applies an offset to the pitch of the note. Positive values offsets the pitch up, negative values down. | cents | 0 | (-inf, +inf) | [-100, 100] |
| `A`  | Adds tremolo to the note based on the pitchbend. Negative values flip the envelope. | percentage | 0 | (-inf, +inf) | [0, 100] |
| `gw` | Adds a faked growl to the render. 100 is similar to [death growl](https://en.wikipedia.org/wiki/Death_growl). | percentage | 0 | [0, 100] | [0, 100] |
| `S`  | Mixes a render where the aperiodicity is maxed out. This produces an almost growl-like whispery tone which can complement the growl flag. | percentage | 0 | [0, 100] | [0, 100] |

[^a]: OpenUtau has gender/GEN set as an expression for this flag with range [-100, 100].

[^b]: OpenUtau has breath/BRE set as an expression for this flag with 0 for the default.