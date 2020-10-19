# Readable password generator (library)

Library for **gen**erating a **re**adable **pass**word from an ordered list of words extracted from text. For improved security, numbers and special characters are inserted at random places.

The point is to replace the standard password generation that is very tedious to input manually, with a still very secure but much easier to write password. For the rare occasion where you have to input it manually, like on a smartphone you're not syncing them to. It also makes for some interesting passwords, depending on what you choose to use as source.

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

## Changelog

### Version 1.1.1 - 2020-10-19

- Fix spelling.

### Version 1.1.0 - 2020-10-19

- Convert from a binary crate into a library crate.
- Move out the CLI into its own crate [genrepass-cli](https://github.com/AlexChaplinBraz/genrepass-cli).

### Version 1.0.1 - 2020-10-13

- Switch from clipboard-ext to copypasta-ext, adding support for Wayland clipboard [[PR1]](https://github.com/AlexChaplinBraz/genrepass/pull/1).

## History

This used to be a binary crate that was a port of my own shell script [genrepass.sh](https://github.com/AlexChaplinBraz/genrepass.sh), which I wrote as practice while learning Rust and ended up improving it in various aspects.

Realising the main functionality could be made into a library I extracted it and ended up improving it heavily once again, this time with much better error handling and logical API with good documentation.

The command line application that was once part of this crate is now called [genrepass-cli](https://github.com/AlexChaplinBraz/genrepass-cli).

## Donate

Please do feel free to [support me](https://alexchaplinbraz.com/donate) if I was helpful and you aren't in a tight financial situation.

## License

MIT
