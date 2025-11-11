# Static Assets

This directory contains static assets that are served by the application.

## Directory Structure

```
static/
├── favicon.png           # Application favicon
├── characters/           # Character stock icons
│   ├── captain-falcon.png
│   ├── donkey-kong.png
│   ├── fox.png
│   ├── falco.png
│   ├── marth.png
│   └── ... (all 26 Melee characters)
└── stages/               # Stage icons
    ├── battlefield.png
    ├── final-destination.png
    ├── fountain-of-dreams.png
    ├── pokemon-stadium.png
    ├── yoshis-story.png
    └── dream-land.png
```

## Character Icons

Character icons should be:
- **Format**: PNG with transparency
- **Size**: 64x64px or 128x128px (will be scaled to fit)
- **Naming**: lowercase-with-dashes (e.g., `captain-falcon.png`, `ice-climbers.png`)
- **Source**: [Melee Stock Icons](https://www.ssbwiki.com/Category:Stock_icons_(SSBM))

### Character ID Mapping

```typescript
0  = Captain Falcon   → captain-falcon.png
1  = Donkey Kong     → donkey-kong.png
2  = Fox             → fox.png
3  = Game & Watch    → game-and-watch.png
4  = Kirby           → kirby.png
5  = Bowser          → bowser.png
6  = Link            → link.png
7  = Luigi           → luigi.png
8  = Mario           → mario.png
9  = Marth           → marth.png
10 = Mewtwo          → mewtwo.png
11 = Ness            → ness.png
12 = Peach           → peach.png
13 = Pikachu         → pikachu.png
14 = Ice Climbers    → ice-climbers.png
15 = Jigglypuff      → jigglypuff.png
16 = Samus           → samus.png
17 = Yoshi           → yoshi.png
18 = Zelda           → zelda.png
19 = Sheik           → sheik.png
20 = Falco           → falco.png
21 = Young Link      → young-link.png
22 = Dr. Mario       → dr-mario.png
23 = Roy             → roy.png
24 = Pichu           → pichu.png
25 = Ganondorf       → ganondorf.png
```

## Stage Icons

Stage icons should be:
- **Format**: PNG (transparency optional)
- **Size**: 320x180px (16:9 aspect ratio)
- **Naming**: lowercase-with-dashes
- **Source**: [Melee Stage Screenshots](https://www.ssbwiki.com/Super_Smash_Bros._Melee#Stages)

### Stage ID Mapping

```typescript
2  = Fountain of Dreams    → fountain-of-dreams.png
3  = Pokémon Stadium       → pokemon-stadium.png
8  = Yoshi's Story         → yoshis-story.png
28 = Dream Land            → dream-land.png
31 = Battlefield           → battlefield.png
32 = Final Destination     → final-destination.png
```

## Usage in Code

```typescript
// Character icon
const characterIcon = `/characters/${getCharacterSlug(characterId)}.png`;

// Stage icon
const stageIcon = `/stages/${getStageSlug(stageId)}.png`;
```

## Fallbacks

If an icon is missing, the app will fall back to:
- **Characters**: Colored circle with first letter
- **Stages**: Text name only

## License

All images should be properly licensed. Melee assets are © Nintendo.
For personal/non-commercial use only.

