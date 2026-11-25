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

- In progress, 21 November about 5:50 pm: Trinity (White) - Hobbes 4.0.
  ccrl.live shows evals from White's side. Move 10 (g4): Trinity -0.62,
  Hobbes -0.09; move 16 (Qe1): Trinity -0.69 at d22, Hobbes -0.14 at
  d32. Speed: Trinity 1.22M and 1.20M nps, Hobbes about 0.5M. Again
  Trinity rates its own position about half a pawn lower than its
  opponent does (as in #24): a pattern to check in forensics (is
  Trinity's eval too pessimistic about its own side, or is it simply
  seeing more?). Screenshots in games/ccrl.

## Games 46, 53, 60 (22 November): three draws; score 4.5/8 (56%), unbeaten
PGN evals are each engine's own view (positive = good for the mover).
- #46 Trinity - Hobbes, draw at move 142: the queen ending. Trinity
  -3.06 from move 37 to 64 while Hobbes said +0.30 throughout; Trinity
  then drifted -2.47, -1.88, -1.07, -0.87, -0.2, 0.00 as the checks went
  on. Hobbes was right: a draw. Confirms the urgent note of 21 November.
- #53 Trinity - PZChessBot, draw at move 79: the OPPOSITE direction.
  Trinity +1.2 to +1.5 for its own side (moves 9-20) and +0.8 to +0.9
  (moves 24-57) while PZChessBot rated White about +0.55 and later +0.11
  to 0.00. Draw.
- #60 Trinity - ZigQueen, draw by perpetual at move 53: both agreed
  Black was better; Trinity held.
Pattern across #24, #46, #53 (and the live -2.2 to -3.2 readings):
Trinity's evals are larger IN SIZE than its opponents', in both
directions, in positions that end drawn. That points less to "pessimism"
and more to an over-scaled eval (centipawn scale too large), or to a
network that does not shrink evals toward 0 in drawish endings.
Practical effects: (1) CCRL draw adjudication needs both evals below
0.20, so inflated evals make games run very long (#46: 142 moves);
(2) any eval-based decisions in search (margins, time) work on an
inflated scale; (3) resign adjudication could misfire if an opponent
agreed. Main (wdl25-900, WDL 0.25) should be checked on these same
positions: does it shrink drawish endings toward 0?

## Game 67 (22 November): Trinity - Heimdall 1.5.2, draw at move 89; score 5/9, unbeaten
Black (Heimdall) was slightly better from about move 23 by both
engines' view (Heimdall +0.4 to +0.6 for itself). From move 60 Trinity
evaluated the minor-piece ending as 0.00 throughout, while Heimdall kept
+0.5 to +0.6 for 30 moves. Trinity ended it with 88.Nd5+ exd5 89.Bxd5,
leaving bishop against bishop and knight with no pawns: no mating
material, a dead draw (Trinity depth 124, 0.00). Here Trinity's
judgement was the more accurate one: a counter-example to the
"Trinity overstates drawish positions" question of #46/#53.
Tournament table after game 69 (advisor's tally): ZigQueen 7/9,
Crownd 6/8, Hobbes 5.5/8, Cinder 5/8, Heimdall 5.5/9, Trinity 5/9
(unbeaten, one of four), Ayano and ngn 5/9, PZChessBot 4.5/8. Opponents'
CCRL 1CPU ratings: Hobbes 4.0 3612, Cinder 0.6.1 3608, Heimdall 1.5.2
3570, ZigQueen 6.4.0 3566: Trinity is performing above its 3522.

## Game 74 (22 November): Trinity - NARC Genesys 5.1, draw at move 67; score 5.5/10, unbeaten
NARC won a pawn (move 21-26) and outplayed Trinity into a rook ending,
rating its own side +4.6 to +5.2 from move 36 to the end. Trinity said
-0.2 and then 0.00 from move 44 (depth 31-59) and was right: the game
ended in a drawn 5-piece position (rook vs rook and pawn) at move 67.
NARC's author said in the event chat that its network was trained only
on self-play without tablebases and over-values extra material in drawn
endings; this game shows it. Second game in a row where Trinity's
endgame judgement was the accurate one (after #67), against two where
it ran high (#46, #53). Fine material for the tablebase-truth
measurement planned after the freeze.

## Games 81, 90, 99 (23 November): score 7.5/13, still unbeaten
- #81 Trinity - Ember 1.4.0, WIN at move 50. Both engines agreed White
  was better out of the book (Trinity +1.2, Ember -0.9); Trinity
  castled long, won material on the d-file and pushed the e-pawn home.
- #90 Erebus 2.2.7 - Trinity, draw at move 106. Rook ending in which
  Trinity was a pawn DOWN (White f2/f3, f4, h3 against f5, g6 from move
  47) yet rated its own side +0.84 to +1.01 for about 40 moves (depth
  55-66), while Erebus said 0.00 throughout; a drawn ending, so Erebus
  was right. Another case of Trinity's endgame eval running high, like
  #46 and #53 (not checked with a referee engine yet).
- #99 CeylonDemon 3.3 - Trinity, draw at move 183. CeylonDemon won a
  pawn and pushed a passed a-pawn to a7; it rated itself +2.1 for 100
  moves, Trinity -1.5, and Trinity held a fortress (two rooks each)
  until both said 0.00. Trinity's judgement was the closer one.
Endgame-eval tally for the tablebase-truth measurement after the
freeze: ran high in #46, #53, #90; accurate where the opponent ran high
in #67, #74, #99. Game files: games/ccrl/16066-81/90/99-*.pgn.

## Games 108 and 117 (23 November): two losses; 7.5/15 so far (the event continues: game 144 vs StockDory 0.4 was live on 24 November)
- #108 Cinder 0.6.1 (CCRL 3608) - Trinity, LOST at move 74. The book
  line (Najdorf with h3/g4) left Trinity worse from the start: -1.1 at
  its first own move, Cinder +0.6. A long squeeze; Trinity's evals were
  consistently realistic (about -1.2 to -2.0 while Cinder said +0.7 to
  +1.1), so no misjudgement stands out. Lost to a stronger engine on a
  bad book position.
- #117 crownd - Trinity, LOST at move 44. Equal (0.00 both) to move 15.
  15...Bg6 (Trinity -0.01) allowed 16.Bxe4 and Trinity dropped to -0.6.
  Then the key point: from 21...hxg4 to 25...Nf3+ Trinity rated its
  kingside attack 0.00 to -0.19 (it expected a draw, probably by
  perpetual check) while crownd said +1.5 to +2.9; the attack ran out
  and crownd won. Candidate for forensics with the referee engine:
  a search/horizon misjudgement (moves 20-25), not an endgame eval one.

## Games 135 and 144 (24 November): two draws
- #135 Trinity - ngn 0.4.0, draw at move 41 by repetition. Level from
  the book (both about 0.00 throughout); Trinity's small edges (+0.4 to
  +0.5 around moves 15-19) never became anything.
- #144 Trinity - StockDory 0.4, draw at move 62. Sharp Sicilian
  (6.Bc4, 7.f4, 10.g4); Trinity's pawn reached f7 and both rated it
  0.00 from move 42; both sides queened (a8 and g1) into a dead-level
  queen ending. Both engines' evals agreed almost all game: nothing
  for forensics. Trinity searched 1.1-1.2M nodes/s against StockDory's
  2.2M on Graham's machine.
- Game 126 (Trinity's, between 117 and 135) not yet received.

## Game 152 (24 November): clrsrc 1.4.0 - Trinity, draw at move 65; 9/18 (2 wins, 14 draws, 2 losses)
Trinity (Black) came out of the book better: +0.8 to +1.2 by its own
count, and clrsrc agreed (-0.7 to -0.9) for moves 9-17. The edge went
in two moves: after 17...Rxa1 18.Rxa1 Rd6 Trinity fell from +1.06 to
+0.38, then +0.19 after 19.f3 Rd3, and the game was level from move 20.
Candidate for forensics (referee engine): was 18...Rd6 (or 16...Rfxd8 /
17...Rxa1) where the advantage was lost? From move 59 a rook-and-knight
each with no pawns: dead draw. Trinity said 0.00 there throughout,
clrsrc +0.1 to +0.24: Trinity the accurate one. Trinity searched only
about 180K nodes/s in that ending (tablebase probing, likely).
Ayman chose game 81 (Trinity - Ember, the win) for next week's post.

## Game 159 (24 November): Valkand 1.1 - Trinity, draw at move 61; 9.5/19 (2 wins, 15 draws, 2 losses)
Level from the book throughout (both engines within +/-0.2 nearly all
game). Trinity held small edges of +0.3 to +0.7 by its own count from
moves 29-41 while Valkand said -0.15; nothing came of them, and the
bishop-and-queen ending was drawn. Unremarkable; nothing for forensics.

## Game 166 (25 November): Hobbes 4.0 (CCRL 3612) - Trinity, LOST at move 86; 9.5/20 (2 wins, 15 draws, 3 losses)
A slow closed game (Four Knights with g3, then a kingside pawn advance by
White). Out of the book Trinity rated itself +0.5 to +0.9 while Hobbes
said about 0.00. From move 21 (21...Ng8, 22...g6, 23...Ngf6, a passive
kingside regrouping) Trinity's own score fell: -0.4, -0.75, -1.55, while
Hobbes still said only +0.4. From move 30 to 50 Trinity said -2 to -5
while Hobbes said +0.4 to +1.1, and from move 53 Trinity -6 to -15
against Hobbes +1.7 to +2.1. Trinity was right that it was losing (it
lost a rook ending with Hobbes's passed a-pawn), but its evals were 4 to
7 times larger than Hobbes's at every stage: the clearest example yet of
Trinity's eval magnitude running high, here in a MIDDLEGAME, not only in
endings (#46, #53, #90 were endings). For the tablebase-truth / eval-
scale measurement after the freeze, and a forensics candidate: was the
21-23 regrouping the mistake, or was the position already worse?

## Games 173, 180, 187 (25 November): one loss, two draws; 10.5/23 (2 wins, 17 draws, 4 losses)
- #173 PZChessBot v8.0 - Trinity, LOST at move 48. A Dutch (Stonewall
  with ...g5) from the book; Trinity rated itself -0.6 at its first own
  move. 14...b5 15.cxb5 cxb5 16.Nxb5 cost a pawn (Trinity -0.74 then
  -1.37/-1.54), and White converted. Evals again larger than the
  opponent's: Trinity -1.5 to -2 when PZChessBot said +0.8 to +1.3,
  -5 to -9 when it said +4 to +6 (about 1.5-2x; smaller than #166's
  4-7x). Referee question: was 14...b5 the mistake?
- #180 ZigQueen 6.6.0 (CCRL 3566, an event leader) - Trinity, draw at
  move 76. Trinity +0.4 to +0.9 by its own count out of the book, ZigQueen
  -0.3 (both agreed Black was a little better); it ran down to a level
  rook ending. A good result against one of the strongest in the field.
- #187 Heimdall 1.5.2 - Trinity, draw at move 21 by repetition. A sharp
  Dutch (1.d4 f5 2.Bg5); both 0.00 from move 13; the queen shuffled
  between g5 and h6 against Black's bishop.
Two of Trinity's four losses (#108 Cinder, #173 PZChessBot) began from
book positions both engines rated clearly worse for Trinity.

## Correction (advisor, 25 November): engines print evals on different scales
Scout measured it (scout/2026-11-25.md, section 2): in games the opponent
WON, Hobbes prints about a quarter of Trinity's number, Patricia and Prune
about half, Bread and akimbo a little more than Trinity. So the "4 to 7
times Hobbes's" reading of #166 and the "1.5-2x" of #173 are mostly the
opponents' own scales, not evidence that Trinity's evals run high; strike
them from the eval-scale question. Trinity's -4 in #166 was right (a score
of 3.5 or more is followed by a loss 95-97% of the time). The endgame
finding of 23 November uses game results, not opponents' numbers, and
stands. Compare evals across engines only after each engine's factor.

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
