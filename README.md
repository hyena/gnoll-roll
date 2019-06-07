# Gnoll Roll

A dice bot for discord.

Gnoll roll was created when some friends using [Sidekick](https://github.com/ArtemGr/Sidekick) had trouble trusting its streaks as 'truly' random and unskewed. While I believe Sidekick is almost certainly unskewed (it's more work to write an unfair roll bot than a fair one and serves no obvious purpose) I can't confirm that as it's closed source. Besides, I wanted some practice writing discord bots and parsers in Rust so this was perfect.

Gnoll Roll uses Rust's [ThreadRng](https://rust-random.github.io/rand/rand/rngs/struct.ThreadRng.html) for its rolling. You can decide for yourself if that's random enough.

## Roll Syntax

Gnoll Roll supports a subset of Sidekick's syntax:

`/gr 1d8 + 4d6` - Roll one octahedron and four hexahedrons.

`/gr 1d20+5 # Grog attacks` - Roll dice with a comment.

`/gr 2d6>=5` - Roll two hexahedrons and take only the ones that turned greater or equal to five (aka difficulty check). Prints the number of successes.

`/gr 4d6=5` - So can this guy roll five?

`/gr 3d10>=6f1` - oWoD roll: rolling *one* is a failure, rolling more failures than successes is a *botch*.

`/gr 1d10>=8f1f2` - Rolling *one* or *two* is a failure.

`/gr 1d20r1` - Roll twenty, reroll on one (because halflings are lucky).

`/gr 3d10!>=8` - nWoD roll: tens explode, eights and up are treated like a success.

`/gr 2d20k1` - Roll twice and keep the highest roll (D&D 5e advantage).

`/gr 2d20k1 + 2` - Roll twice and keep the highest roll, with a modifier (D&D 5e advantage).

`/gr 2d20kl1` - Roll twice and keep the lowest roll (D&D 5e disadvantage).

`/gr 4d6k3` - Roll four hexahedrons and keep the highest three (D&D 5e ability roll).

Notable Gnoll Roll does NOT support saving named dice rolls and some features (e.g. Fudge dice). Since it's open source, though, you're welcome to contribute.

## Should I use Gnoll Roll?

If you just want a discord dice bot you should use Sidekick. It's older, has more features, and will probably be better supported.

If you looking at the source code of your bot is important to you due to paranoia or if you simply want to contribute yourself, you can use Gnoll Roll.

## TODO

This section is just to keep track of what I need to do for a 'release'.

  - [] Parsing methods should return errors and never panic. Two kinds of panics: Parsing error and unexpected
  - [] Limit `count` to something reasonable (512 is a good choice)
  - [] Implement reroll, success and keep rules
  - [] Figure out how to do tests better (mock the RNG probably)
  - [] Write discord integration
  - [] Delete this section
