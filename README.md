# Readable password generator

**Gen**erate a **re**adable **pass**word from an ordered list of words extracted from text. For improved security, numbers and special characters are inserted at random places.

The point is to replace the standard password generation that is very tedious to input manually, with a still very secure but much easier to write password. For the rare occasion where you have to input it manually, like on a smartphone you're not syncing them to. It also makes for some interesting passwords, depending on what you choose to use as source.

Written based on a Computerphile video: [How to Choose a Password](https://youtu.be/3NjQ9b3pgIg).

## Usage

`genrepass [FLAGS] [OPTIONS] <path>`

## Flags

`-C, --capitalise` Uppercase the first character of every word. Makes the password much easier to read, but also slightly less secure due to the predictability of having capitalised words. Still, the highly improved readability makes it worth it to always have it on.

`-c, --clipboard` Copy the generated password/s to clipboard instead of writing to stdout.

`-d, --dont-lower` Don't lowercase at all to keep original casing. Ignores '--force-lower', both manual and automatic.

`-D, --dont-upper` Don't uppercase at all to keep original casing. Ignores '--force-upper', both manual and automatic.

`-f, --force-lower` Force the specified amount of lowercase characters. Gets ignored if '--dont-lower' is also set.

`-F, --force-upper` Force the specified amount of uppercase characters. Gets ignored if '--dont-upper' is also set.

`-h, --help` Prints help information.

`-k, --keep-nums` Choose to keep numbers from the source in the password. It will treat blocks of numbers as words, not counting them towards the amount of numbers to insert into the password.

`-X, --randomize` Shuffle the words. Useful if the source text is just a list of words without order anyway and you want to have a different order with each run of the program.

`-r, --replace` Replace the original characters. Instead of inserting the numbers and special characters (which preserves the original words), replace the characters at random positions.

`-V, --version` Prints version information


## Options

`-L, --length <length>` Set the length of the password. Can either be a range like 24-30, which will generate a password between that length, or it can be an exact number like 25 for a password of that exact length. Default: 24-30

`-l, --lower <lower>` Amount of lowercase characters. Can take either a range like 2-4 or an exact amount like 2. If there are no lowercase characters, the '--force-lower' flag is turned on automatically to decapitalise up to the specified amount of alphabetic characters. But if there's at least one lowercase character there won't be any decapitalisation unless '--force-lower' is turned on manually. Default: 1-2

`-R, --resets <max-resets>` Amount of times to try generating password before truncating. If the range is too small or an exact number, it'll be harder to get a fitting set of words, so the word selection will restart if the password exceeds the maximum length. But since it would keep looping if it doesn't find the right length it needs a way to stop, which in this case is simply truncating the password to the maximum length. Default: 10

`-n, --num <num>` Amount of numbers to insert. Can take either a range like 2-4 or an exact amount like 2. Doesn't take into consideration the amount of numbers already in the password if '--keep-nums' is activated. Default: 1-2

`-p, --pass-amount <pass-amount>` Amount of passwords to generate. Each password comes on a new line. Useful for providing a list of passwords to choose from. Default: 1

`-s, --special <special>` Amount of special characters to insert. Can take either a range like 2-4 or an exact amount like 2. Default: 1-2

`-S, --chars <special-chars>` The special characters to insert. Non-ASCII characters are not supported. Default: ^!(-_=)$<[@.#]>%{~,+}&*

`-u, --upper <upper>` Amount of uppercase characters. Can take either a range like 2-4 or an exact amount like 2. If there are no uppercase characters, the '--force-upper' flag is turned on automatically to capitalise up to the specified amount of alphabetic
characters. But if there's at least one uppercase character there won't be any capitalisation unless '--force-upper' is turned on manually. Default: 1-2

## Args

`<path>` Path to text file or directory with text files to source words from.

In case of a directory, it recursively parses every file inside it while ignoring non-plaintext files and following links.

Accepts UTF-8 characters, but translates them to ASCII for use in the password. So if a word in another language is encountered, it will be transformed into a kind of phonetic spelling in ASCII, and if an emoji is encountered, it will be translated into its meaning, for example, :D would become 'grinning'.

## Builds for other systems

I'm sure this program can be compiled for other platforms like macOS, but I didn't do it because I don't have the capability to test them. If anyone is willing to help with that, I'd appreciate it.

## History

This is a port of my own shell script [genrepass.sh](https://github.com/AlexChaplinBraz/genrepass.sh). I've written it as practice while learning Rust and ended up improving it in various aspects.

## License

MIT
