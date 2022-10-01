# Readable password generator (library)

Library for **gen**erating **re**adable **pass**words from an ordered list of words extracted from text.
For improved security, numbers and special characters are inserted at random places.

The point is to replace the standard password generation that is very tedious to input manually,
with a still very secure but much easier to write password.
For the rare occasion where you have to input it manually, like on a smartphone you're not syncing them to.
It also makes for some interesting passwords, depending on what you choose to use as source.

Written based on a Computerphile video: [How to Choose a Password](https://youtu.be/3NjQ9b3pgIg).

## Some examples of possible passwords

```
Hemanag6estogetami~ssion
ModsSo$ThatPeopleAre7AbleTo
5Th~ingsT<0hatThrowYouOff
Fr68omTheseMeth&odsY#ouCanT
AtLeastEvo]keA,Menta3lImage
CanJus6tDownloadItB^utTheBe]ne3fit
TheP~edanticTutorialsOfE1veryS^i7ngle
therearetoomAny{volum9estob(eadaptedi9ntoThe
```

## Documentation

Can be found at [docs.rs/genrepass](https://docs.rs/genrepass).

## History

This used to be a binary crate that was a port of
[my own shell script](https://github.com/AlexChaplinBraz/shell-scripts/tree/master/genrepass),
which I wrote as practice while learning Rust and ended up improving it in various aspects.

Realising the main functionality could be made into a library,
I extracted it and ended up improving the program once again,
this time with much better error handling and logical API with good documentation.

The command line application that was once part of this crate is now called
[genrepass-cli](https://github.com/AlexChaplinBraz/genrepass-cli).

## Donate

Please consider supporting me through [donate.alexchaplinbraz.com](https://donate.alexchaplinbraz.com/?project=2)
to motivate me to keep working on this project.

## License

MIT
