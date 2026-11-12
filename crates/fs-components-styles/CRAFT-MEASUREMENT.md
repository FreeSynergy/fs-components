# Craft: measurement sheet

Measured on the two public screenshots of the ArtCraft Launcher (project `storytold/craft-launcher`, MIT OR Apache-2.0):
`docs/screenshots/artcraft-launcher-grid.png` (dark) and `artcraft-launcher-table.png` (light), each 1180 x 900 with an 8 px
margin of window shadow (content 1164 x 884). Only numbers were taken, nothing was copied (no picture, no font, no text).
Colours are the most frequent pixel value of the part.

## Colours (dark)
| Part | Value |
|---|---|
| Ground of the window, the bar, the tiles | `#121316` |
| Deep ground (search field, badges) | `#060607` |
| 1 px line (bar, tile head, badge, button, tab, progress track) | `#333436` |
| Stronger line between columns of tiles | `#4F5051` |
| Text: white / secondary / dim (footer, the line under a summary) | `#F2F1EE` (titles `#FFFFFF`) / `#989998` / `#666768` (we use `#8A8B8C` for 4.5:1) |
| Badge text: update / offer / installed | `#FFC14D` / `#74AAFF` / `#3FD6B8` |
| Number box fill (accents 1 to 7) | `#2F7BF5`, `#E8573F`, `#8B5CF6`, `#F2A516`, `#12A58A`, `#E0368F`, `#7BB51C` |
| Second half of a name (lighter accents) | `#7FB0FF`, `#FF8A70`, `#B59BFF`, `#FFC14D`, `#3FD6B8`, `#FF7FBF`, `#A5D94A` |
| Progress fill | the accent of the tile (`#8B5CF6` measured) |
| Primary button | fill `#F2F1EE`, text `#121316` (white button, black capitals) |
| Outline button | line `#6C6B6C`, text `#F2F1EE` |

## Colours (light, from the table picture)
Ground `#F2F1EE`, field `#E9E8E4`, line `#CECDCB`, strong line `#A5A5A3`, text `#101014`, secondary `#6F6E6F`; badge text amber `#8A5800`, blue `#1A5BC4`.

## Sizes (pixels, content origin at 8,8)
| Part | Measure |
|---|---|
| Top bar | 47 high, 1 px line below (y 55) |
| Filter row | tabs and fields 34 high (y 74 to 108), 32 px below the bar |
| Tile | 375 wide, 279 high in the picture; lines shared (one 1 px line between two tiles) |
| Tile head | 39 high with a 1 px line below; number box 31 x 22 (22 high, at least 31 wide) |
| Tile body padding | 20 |
| Logo | 56 x 56 (rounded about 12) |
| Button | 34 high; "..." menu square 36 x 34; pause and cancel 32 x 32 |
| Badge | 20 high (4 px above and below the text, 8 left and right), 1 px line |
| Progress | track 1 px line colour, 4 high; bar fills the share |
| Footer | 33 high, 1 px line above |
| Corners | 0 everywhere (logo only about 12) |

## Type (cap height measured, size derived at 0.73 em)
| Part | Cap height | Our size / face |
|---|---|---|
| Tab, badge, button, link, footer: capitals, mono, spaced | 8 to 9 | 11, JetBrains Mono, bold (tabs, buttons) or regular (links, footer), spacing 0.06 to 0.08 em |
| Section title `01 / CLASSIC APPS`, count | 9 to 11 | 11 dim |
| Tile name (two tones: white, then accent) | 18 incl. descender | 18, sans, bold |
| Summary under a name | 13 | 13, sans, `#989998` |
| Date and version lines | 13 | 13, sans, `#989998` |
| Brand, top tabs | 14 / 9 | 14 bold caps / 11 |

The measured face of the capitals is narrower than JetBrains Mono (advance about 0.49 em against 0.6 em), so our labels are a little
wider at the same cap height; the cap height was kept.

## What is ours, not measured
The accent of a package is chosen by a hash of its name (stable); the progress of a job is the phase (the store reports phases, not
bytes); the dark colour of dim text is lifted from `#666768` to `#8A8B8C` for a contrast of 4.5:1.
