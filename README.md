# game-of-life

Introducing GoLaaS: **G**ame **o**f **L**ife **a**s **a** **S**ervice

- [game-of-life](#game-of-life)
  - [Formats](#formats)
    - [Text `*.txt`](#text-txt)
    - [SVG `*.svg`](#svg-svg)
  - [API](#api)
    - [`GET /`](#get-)
    - [`GET /:game(.txt|.svg)`](#get-gametxtsvg)
      - [Query Parameters](#query-parameters)
      - [Headers](#headers)
    - [`POST /:game`](#post-game)
      - [Query Parameters](#query-parameters-1)
  - [FAQ](#faq)

## Formats

### Text `*.txt`

```console
you@local:~$ curl 'https://game-of-life.reb.gg/fig8.txt'
............
............
............
......###...
......###...
......###...
...###......
...###......
...###......
............
............
............
```

### SVG `*.svg`

<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://game-of-life.reb.gg/fig8.svg?next=true&fill_color=white&stroke_color=black">
    <source media="(prefers-color-scheme: light)" srcset="https://game-of-life.reb.gg/fig8.svg?next=true">
    <img alt="figure 8 game of life board" src="https://game-of-life.reb.gg/fig8.svg?next=true">
  </picture>
</div>

(with `?next=true`, changes on refresh!)

## API

### `GET /`

Redirects to this repository!

### `GET /:game(.txt|.svg)`

Render your existing game as txt or svg!

#### Query Parameters

| param | usage | default |
| - | - | - |
| `next` | iterate to the next generation | `false` |
| `alive` | (txt) char for the alive cell | `#` |
| `dead` |  (txt) char for the dead cell | `.` |
| `separator` | (txt) char for the line separator | `\n` |
| `cell_size` | (svg) width/height of the rendered cell | `20` |
| `stroke_width` | (svg) width of the stroke | `2` |
| `stroke_color` | (svg) color of the stroke | `white` |
| `fill_color` | (svg) color of the alive cells and text | `black` |

#### Headers

| header | example | description |
| - | - | - |
| `x-life-generation` | 0 | generation iteration |
| `x-life-delta` | 0 | changed cells in this generation |

<details> <summary> ℹ️ Examples </summary>

```console
you@local:~$ curl 'https://game-of-life.reb.gg/fig8.txt?alive=%E2%AC%9C&dead=%E2%AC%9B'
⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛
⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛
⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛
⬛⬛⬛⬛⬛⬛⬜⬜⬜⬛⬛⬛👾
⬛⬛⬛⬛⬛⬛⬜⬜⬜⬛⬛⬛
⬛⬛⬛⬛⬛⬛⬜⬜⬜⬛⬛⬛
⬛⬛⬛⬜⬜⬜⬛⬛⬛⬛⬛⬛
⬛⬛⬛⬜⬜⬜⬛⬛⬛⬛⬛⬛
⬛⬛⬛⬜⬜⬜⬛⬛⬛⬛⬛⬛
⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛
⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛
⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛
```

```console
you@local:~$ curl 'https://game-of-life.reb.gg/foo.svg'
<svg xmlns="http://www.w3.org/2000/svg" width="240" height="260"><rect x="120" y="60" width="20" height="20" fill="black" stroke="white" stroke-width="2"/><rect x="140" y="60" width="20" height="20" fill="black" stroke="white" stroke-width="2"/><rect x="160" y="60" width="20" height="20" fill="black" stroke="white" stroke-width="2"/><rect x="120" y="80" width="20" height="20" fill="black" stroke="white" stroke-width="2"/><rect x="140" y="80" width="20" height="20" fill="black" stroke="white" stroke-width="2"/><rect x="160" y="80" width="20" height="20" fill="black" stroke="white" stroke-width="2"/><rect x="120" y="100" width="20" height="20" fill="black" stroke="white" stroke-width="2"/><rect x="140" y="100" width="20" height="20" fill="black" stroke="white" stroke-width="2"/><rect x="160" y="100" width="20" height="20" fill="black" stroke="white" stroke-width="2"/><rect x="60" y="120" width="20" height="20" fill="black" stroke="white" stroke-width="2"/><rect x="80" y="120" width="20" height="20" fill="black" stroke="white" stroke-width="2"/><rect x="100" y="120" width="20" height="20" fill="black" stroke="white" stroke-width="2"/><rect x="60" y="140" width="20" height="20" fill="black" stroke="white" stroke-width="2"/><rect x="80" y="140" width="20" height="20" fill="black" stroke="white" stroke-width="2"/><rect x="100" y="140" width="20" height="20" fill="black" stroke="white" stroke-width="2"/><rect x="60" y="160" width="20" height="20" fill="black" stroke="white" stroke-width="2"/><rect x="80" y="160" width="20" height="20" fill="black" stroke="white" stroke-width="2"/><rect x="100" y="160" width="20" height="20" fill="black" stroke="white" stroke-width="2"/><text x="50%" y="255" font-family="monospace" font-size="12" fill="black" dominant-baseline="center" text-anchor="middle">t = 0, Δ = 0</text></svg>
```

</details>


### `POST /:game`

Create a new game. Submit the game as a raw body.

#### Query Parameters

| param | usage | default |
| - | - | - |
| `alive` | char for the alive cell | `#` |
| `dead` |  char for the dead cell | `.` |
| `separator` | char for the line separator | `\n` |

<details> <summary> ℹ️ Examples </summary>

```console
you@local:~$ curl -X POST --data-binary @examples/fig8 https://game-of-life.reb.gg/foo
............
............
............
......###...
......###...
......###...
...###......
...###......
...###......
............
............
............
```

</details>


## FAQ

> Q: How is state persisted?

SQLite, the game "board" is compressed (with [ZSTD](https://github.com/facebook/zstd)) and stored as a blob.

> Q: Where is it hosted?

[Fly.io](https://fly.io)

> Q: Is it toroidal?

No, but feel free to open a PR!
