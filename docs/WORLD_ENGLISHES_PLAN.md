# World Englishes: tongues for your worlds, from six moves

For the VS Code session working in this repo. Written 2026-10-05 from kfman's
lines below. Nothing here is built yet. Rust, terminal UI, no new crates.

## What kfman said (verbatim, in order)

- "hoom" / "i spake tree and did noot enter"
- "we need more interesting english langs for my worlds"
- "lets find trajectories that fit idolectyl like features of language such
  that we can devise a solution from least parts to reorient the rubicks cubes
  of our thoughts"
- "summarize for vs code (rustty as rustty can ttybe)"

## The idea in one line

A seeded generator that takes plain English and gives it back as one of your
worlds' Englishes (Ent-slow, salt-coast, high-court, burrow-cant), built from
six reversible moves, the way every state of a Rubik's cube comes from six
face turns.

## Least parts: the six faces

Each face is one small table, one rule and one strength (0 to 3). Strength 0
leaves the line alone. Plain line for the examples: "I spoke the tree's
language and did not go in."

| face | key | what it turns | example at strength 2 |
|------|-----|---------------|-----------------------|
| WORDS | W | swaps ordinary words for the tongue's own (yes -> aye, language -> tongue, go -> gang) | I spoke the tree's tongue and did not gang in. |
| SOUNDS | S | regular spelling shifts by rule (not -> noot, -ing -> -in', and -> an') | I spoke the tree's language an' did noot go in. |
| ORDER | O | moves the object or the verb first, adds tags ("..., did I not?") | The tree's language I spoke, and in I did not go. |
| FORMS | F | old or odd word forms (spoke -> spake, goes -> goeth, a-going) | I spake the tree's language and went not in. |
| MARKS | M | particles the speaker scatters at the start, middle or end (hoom, hm, mind, aye) | Hoom. I spoke the tree's language, hm, and did not go in. |
| PACE | P | rhythm: repeats, long compounds (tree-tongue), slow ellipses | I spoke the tree-tongue... the tree-tongue... and did not go in. |

**Every move records what it replaced** (the span it wrote and the plain text
under it). That is what makes a move reversible, and the six of them together
the "least parts": no tongue needs a seventh kind of change.

## A tongue is a move string

Written like cube notation: `W2 S1 O0 F3 M2 P1 #seed`.

- Two tongues are close when their strings are close (add up the six
  differences).
- An **idiolect** (one speaker's own way of talking) is the tongue's string plus
  a small personal offset seeded from the speaker's name: each face moves by
  -1, 0 or +1, kept inside 0..3.
- Register (talking to a lord or to a friend) is one more nudge, +1 or -1 on
  FORMS and MARKS.

Examples, made by hand to show the feel (the generator's will differ):

- Plain: I spoke the tree's language and did not go in.
- Ent, `W1 S0 O1 F2 M3 P3`: Hoom, hm. Tree-tongue I spake... and went not in. Hoom.
- Salt coast, `W2 S2 O0 F1 M1 P0`: Aye, I spake the tree's tongue an' didnae gang in.
- High court, `W1 S0 O2 F3 M0 P1`: The tongue of the tree spake I, and entered not.

## Trajectories

How a tongue changes as you move through a world. Three curves, each giving a
multiplier from 0 to 1 that scales the base move string:

- **Space.** Dialects differ fast over short distances and slowly over long
  ones: growth is sublinear, close to a logarithm.
  `space(d) = min(1, ln(1 + d / d0) / ln(1 + D / d0))`, with `d0` about a day's
  walk and `D` the width of the world.
- **Time.** A change in a language runs as an S-curve: slow, then fast, then
  slow. `time(y) = 1 / (1 + exp(-k * (y - y0)))`.
- **Person.** The idiolect offset and the register nudge above.

A speaker's tongue at a place and year:
`moves = round(base * space(d) * time(y)) + offset`, each face kept in 0..3.
The **trajectory** is the path those six numbers take as `d` or `y` changes:
walk away from the capital and watch the move string drift.

Sources, recalled and NOT checked in this session (check before quoting): the
sublinear distance curve is from dialectometry (Seguy, 1971; measured across
dialect atlases by Nerbonne, 2010); the S-curve of change is Kroch's (1989).

## Reorienting: the gloss

`gloss(spoken) -> plain` undoes the recorded moves in reverse order. Two uses:

1. In the panel, **G** shows the plain line under the spoken one, so anyone can
   read every tongue (useful to as many people as possible).
2. A test proves the round trip for every sample, family and strength.

## How it fits this crate

The crate's own pattern carries it (see `src/procgen/mod.rs`, "Adding a
generator"):

- `src/procgen/generators/tongue.rs`: `TongueGen`, a unit struct implementing
  `Generator`, re-exported from `generators/mod.rs`.
- `default_params`: six `Param::int("Words", 1, 0, 3, 1)` style sliders (one a
  face), plus `Param::int("Distance", 0, 0, 10, 1)`,
  `Param::int("Era", 5, 0, 10, 1)` and
  `Param::choice("Family", &["Ent", "Salt coast", "High court", "Burrow"])`.
  The panel draws them with no UI code (`params.rs`).
- `generate(params, seed)` returns a new `ObjectData::Text(TextData)`:
  `lines: Vec<(String, String)>` (plain, spoken) and `moves: String`.
- Per-face random streams with the crate's `mix(seed, face)`, so turning one
  face never reshuffles another.
- Tables are Rust `const`s for now; Phase 3 of `GENERATORS_PLAN.md` moves them
  to asset files like the item tables.
- `world/kind.rs`: a new `ObjectKind::Tongue` mapped to `TongueGen` (the
  `match` will not compile until it is decided).
- `ui/panel.rs`: a text preview for `ObjectData::Text`, plain | spoken in two
  columns, **G** toggles the gloss row. Keys only, like the rest of the panel.
- Add `TongueGen` to `procgen::tests::all()` so the shared determinism test
  covers it.

## Tests (the definition of done)

1. Every face at 0: the output is the input.
2. Same params and seed: the same output.
3. `gloss(spoken) == plain` for every sample line, family and strength 0..3.
4. One face at a time changes only its own feature (ORDER never respells a
   word; SOUNDS never moves one).
5. Output is printable ASCII (plus the apostrophe): safe in any terminal.
6. The curves: `space(0) == 0`, `space(D) == 1`, both curves never go down,
   `time(y0) == 0.5`.

## Order of work

1. `tongue.rs` with WORDS and MARKS (the smallest visible win), the gloss, and
   tests 1 to 3.
2. SOUNDS and FORMS (word-level rules).
3. ORDER and PACE (sentence-level, the hardest).
4. Trajectories: the Distance and Era sliders.
5. The panel preview and G.

Later: NIMERON's worlds already take a mood and a lexicon from a system's name;
they can call the same pure function.

## Not decided (kfman's to say)

- The families' names and feel (the four above are placeholders).
- Whether a tongue belongs to a place (a hex, a town) or to a people.
- Word tables written by hand, or grown from the ~2,000-word defining
  vocabulary.
