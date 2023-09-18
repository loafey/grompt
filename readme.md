# Grompt
A very simple git status prompt.
Simply call `grompt` to list the git status of the current repo.


Example using `grompt -s -i` in my nushell prompt:

![example](example_pic.png)
(Note: the default icons use NerdFonts (FiraCode Nerd Fonts in the example), if you prefer to use something else (emojis or textfor example), you can simply override them using `-o`!)

## Future work
* Add the option to color more then the icon
* Could refactor the code to make it prettier, as the current version was written under 2 hours :))

## Options
```
Usage: grompt [OPTIONS]

Options:
  -p, --path <FILE>
          The folder to check the git status of [default: .]
  -P, --parentheses
          Show parentheses around the output
  -s, --square-brackets
          Show square brackets around the output
  -d, --dirty-string <STRING>
          Show a custom string when a repository is dirty [default: *]
  -i, --icon
          Show icons representative of your remote
  -E, --error
          Print errors to `stderr` instead of silently exiting
  -o, --icon-override <STRING|STRING|U8,U8,U8?>
          Add custom icons for your own git hosts, alternatively override the built in-ones. Add input `-o "git@|<STRING>", to replace the icon for all `git@` remotes. Use the option multiple times for multiple icons, `-o "git@|<STRING>" -o "https://github.com|<STRING>"` etc. Optionally you can add three bytes after to add a color to the icon
  -c, --icon-color
          Enables the use of custom icon colors
  -h, --help
          Print help
  -V, --version
          Print version
```