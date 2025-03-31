# A toolset to make visualisations for [canvas](https://github.com/osucord/canvas)

```
Usage: canvas.exe <COMMAND>

Commands:
  timelapse         Render a timelapse video of the canvas
  virginmap         Render a timelapse video of the canvas, highlighting which pixels haven't been overridden
  agemap            Render a timelapse showing the age of each pixel
  heatmap           Render a heatmap of the canvas
  usermap           Render a usermap of the canvas, showing who placed each pixel
  singleplace       Render the canvas, without placing pixels over drawn pixels
  singleplayer      Render one canvas per user, showing only the pixels they placed.
  longsession       Show a list of the longest sessions, with a max pause of X seconds.
  currentpixels     Make a leaderboard counting only the pixels still on the canvas.
  maincontributors  List the amount of people that were placed most of X% of the pixels
  help              Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

An example database, which was used for our event, can be found [here](https://files.catbox.moe/owoch3.db)
To use this database, make a ".env" file at the root of the repository, containing:
```yaml
DATABASE_URL=sqlite://PATH/TO/DATABASE.db
```
or set it as an environment variable instead.
