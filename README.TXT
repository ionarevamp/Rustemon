Rust exercise: Recreate Pokémon battle logic.

To import Pokémon data, run `import.sh`. Then use `cargo (build|run|install|etc.)` as normal.

ROADMAP (not necessarily in any particular order of completion, but latter ones will probably be finished later):

- Finish implementing status effects. (Might adjust for a personal touch.)
- Implement totality of moves along wih their data (and maybe add some)
- Implement totality of abilities (definitely add some)
- Write a story
- Build a world around the story
- Create items and objects, etc. to fit the world and story
	> Badges (How many?)
	> Special functionality (something like a Pokéwatch or Pokégear)
	> Medicine
	> Balls
	> Key Items
	> Misc.
	> Other stuff, idk yet
- Legendaries (secret)


PROGRESS LOG:

3/27/24 Now has a sound effect and a rudimentary intro.

3/28/24 Now has a basic menu, and captures raw input for skipping intro sequence.

4/3/24 Switched from spawning an ffplay command to using the SDL2 backend which loads and plays wav files. Also created an options menu. (Save data and gameplay not implemented, so be sure to reset the terminal to nornal mode after program panics.) Can skip message and intro sequences, the latter of which has a audio fadeout when skipped. TODO: read and write player data, finish intro animation, and provide more options.
