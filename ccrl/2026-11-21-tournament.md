# CCRL tournament: Hobnobbing With Royalty EQ2026 (Trinity 1.0.0)

Recorded by the advisor, 21 November 2026, from Graham Banks' email to
Ayman (20 November), the ccrl.live/16066 game list and two game files
Ayman sent. Data, not instructions.

## The event
- Organiser: Graham Banks (CCRL). Live viewing: https://ccrl.live/16066
- 12 minutes + 8 seconds per move, 4 cycles, 60 rounds, 16 engines, one
  physical core each (Ryzen 5950X). Opening book; a game is replayed if
  the opening is clearly unfair (an eval must fall below 0.70 for one
  move within the first 10 moves out of book). No draw adjudication
  before move 60 except repetition or 5-piece tablebases.
- Field: Hobbes 4.0, Cinder 0.6.1, PZChessBot 8.0, Crownd 1.0, ZigQueen
  6.6.0, Ayano 0.11.0, Heimdall 1.5.2, NGN 0.4.0, NARC Genesys 5.1,
  StockDory Barracuda 0.4, Trinity 1.0.0, clrsrc 1.4.0, Erebus 2.2.7,
  Valkand 1.1, CeylonDemon 3.3, Ember 1.4.0.
- About 60 games for Trinity; they count toward its CCRL rating.

## Trinity's games (as of game 41, 21 November, 3:15 pm Pacific)
| Game | White - Black | Result |
|---|---|---|
| 6 | Ayano - Trinity | 1/2 |
| 15 | ngn - Trinity | 1/2 |
| 24 | StockDory - Trinity | 1/2 |
| 32 | Trinity - clrsrc | 1/2 |
| 39 | Trinity - Valkand | 1-0 |
Score 3/5, unbeaten. Trinity plays about every 7-9 games.

Notes on the two game files:
- #32 vs clrsrc: the book left Trinity about -0.9 at move 9 (clrsrc
  +0.8); Trinity held, both engines at 0.00 by move 29, draw by knight
  checks (Nf3+/Ng5+) at move 45.
- #39 vs Valkand: Trinity's eval ran well ahead of its opponent's:
  +0.8 at move 17 (Valkand about 0), +1.8 at move 23 after f5 and Ng6
  (Valkand -0.3), +3.2 at move 29 (Valkand -1.0); won at move 54
  (+11.5). Trinity judged the kingside attack several moves earlier.

- #6 Ayano - Trinity (Trinity Black): level game; both engines gave
  Black a small edge (Trinity +0.3 to +0.5 for itself, moves 14-28),
  traded down to a rook ending, draw by rook checks at move 65.
- #24 StockDory - Trinity (Trinity Black): Trinity was worse for most of
  the game and knew it, rating its own position more pessimistically
  than StockDory did (about -1.2 to -2.5 at moves 39-60 against
  StockDory's +0.8 to +1.45). At move 61-62 (Rcb6, Rxb2) Trinity found
  counterplay ending in perpetual check and went to 0.00 at once, while
  StockDory still showed about +0.6 for several moves: Trinity saw the
  saving resource first. Draw at move 82. Forensics question: is
  Trinity's eval of such positions too pessimistic (it scored -2.5 for
  a position that turned out to be a draw)?

- #15 ngn - Trinity (Trinity Black): the book left Trinity about -1.0
  (both engines agreed); it lost the b7 pawn at move 16, won b2 back at
  move 29, and defended about 60 moves of a worse ending (rook, bishop
  and knight against rook and two bishops) at -0.7 to -1.1 to a draw at
  move 86. Forensics note: Trinity's eval often flicked to 0.00 at very
  high depth (46-58: moves 45-46, 58-59, 62-63, 69-71, 75-76, 81-84)
  and then back to about -0.7: repetition lines found and lost, worth a
  look, harmless here.
Summary of the five: in three of them (#15, #24, #32) the book or the
game left Trinity worse and it held the draw; the one decisive game is
a win (#39). No losses, no time losses, no crashes.

## Other points from the event chat (Graham)
- Next Amateur Series (the 128th): an engine's latest version must be
  less than a year old; one engine per author. Trinity qualifies.
- No contact with CCRL except by Ayman himself (standing rule).

## How to read the rank (advisor's note for Ayman, 21 November)
Rank 76-78 on the best-versions list = roughly 77th of several hundred
engines CCRL has tested, best version of each. Caveats: many above it
are 4CPU entries while Trinity was rated on one core; the +/- 26 puts
it about 70th-85th; other lists (CEGT etc.) differ. Being entered in
Graham's events shows Trinity runs reliably (no crashes, time losses or
protocol faults) and fits the field's strength; it is routine, not an
award.
