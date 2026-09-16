# azar — brand prompt portfolio (v2: logo-first)

**Jerarquía: el logo/wordmark ES el sujeto; todo lo demás es atmósfera.**
Workflow: generate (Midjourney/Flux/DALL-E) → Magnific (upscale, creativity
baja para no alucinar letras) → si el modelo manglea el texto, componer el
wordmark al final en Figma. Aspectos: social preview 1280x640 · banner
~1200x400 · avatar 1:1.

Negative universal: `watermark, signature, extra fingers, jpeg artifacts`
(el negative de `text, letters` SOLO en prompts donde el texto se compone
después; en los logo-first el texto va EN el prompt).

## Paleta canónica (variar entre estos 4 regimes)
| Regime | Fondo | Tinta | Acento |
|---|---|---|---|
| **fósforo** | negro puro #000 | verde CRT #3ddc84 | ámbar #ffb000 |
| **ámbar** | negro puro #000 | ámbar CRT #ffb000 | blanco cálido |
| **papel** | off-white #f5f2ea | negro tinta | un solo rojo sello #c0392b |
| **github-dark** | #0d1117 | #e6edf3 | azul #58a6ff + verde #7ee787 |

## Tipografías a probar en los prompts
`JetBrains Mono` · `IBM Plex Mono` · `Space Mono` · `Berkeley Mono` style ·
`Departure Mono` (pixel) — siempre lowercase, tracking amplio.

---

# A. WORDMARK — el logo protagonista

## A1. Title sequence — dense bech32 lattice (v3: DENSO, ámbar, sin room)
> Cinematic film still, 2.39:1 anamorphic — the opening frame of a
> cryptography thriller. A pure black CRT screen fills the ENTIRE frame:
> no monitor, no bezel, no room, the phosphor surface IS the world. The
> whole surface is wallpapered with a dense, perfectly aligned square
> lattice of tiny lowercase monospace glyphs from the set
> qpzry9x8gf2tvdw0s3jn54khce6mua7l — exactly one character per cell,
> thousands of cells, tight uniform spacing, running edge to edge with
> no margins. Deep AMBER phosphor (#ffb000) on near-black — warm, never
> green. At the exact center of the lattice, four cells burn far
> brighter than everything else, spelling the word **azar** in the same
> monospace — the only word on screen, wrapped in a halo of phosphor
> bloom and slow afterglow persistence. A few lattice cells elsewhere
> are caught mid-mutation: the previous glyph still visible as a dim
> ghost under the new one, as if the message is still being revealed. A
> faint geometric pattern — thin concentric rounded-square rings —
> barely legible beneath the lattice, structuring the grid around the
> bright word. Rich authentic CRT artifacts everywhere: horizontal
> scanlines with alternating-line flicker, gentle pincushion curvature
> at the frame edges, glass reflection sheen in one upper corner,
> subtle interlace jitter, one random cell decaying mid-flicker,
> vignette pulling the corners into darkness. Dense but perfectly
> ordered — a Bletchley Park wall of intercepted traffic collapsed into
> a single screen. The Imitation Game meets a signal-intelligence
> console: technological, secretive, warm. Fine film grain over
> everything; the bright word is the single focal point, all else is
> living texture.

Variaciones: fósforo blanco-papel frío en vez de ámbar · las 4 celdas
centrales con un anillo geométrico propio · UNA celda del lattice en
rojo tenue (la anomalía). Añadir `characters strictly from
qpzry9x8gf2tvdw0s3jn54khce6mua7l` si el modelo respeta charset (Flux
mejor que MJ). Magnific: creativity ≤ 2 y solo si conserva la legibilidad
del centro; el lattice denso es lo primero que el upscale puede convertir
en sopa.

## A2. Swiss poster (tipografía pura)
> Flat Swiss International Style poster, enormous lowercase word **azar**
> set in JetBrains Mono filling 70% of the width, strict baseline grid,
> one signal color (choose: #3ddc84 green on black / #c0392b red on
> off-white), generous negative space, subtle risograph misregistration
> and paper grain, tiny index line at the bottom margin reading
> `a=29 z=2 r=3`. Nothing else.

## A3. Marca abstracta (el glyph, sin texto)
> Minimalist logo mark on pure black: a single monospace lowercase glyph
> "z" built from a 5x5 pixel grid, one pixel deliberately off (the
> randomness), phosphor green with subtle CRT bloom, enclosed by a thin
> rounded-square outline that is broken exactly where the off pixel
> sits. Flat vector aesthetic, geometric, iconic, works at 16px favicon
> and 512px. No text anywhere.

## A4. Acuñado — el nombre como objeto (letterpress del wordmark)
> Extreme macro of a letterpress strike: fresh metal type blocks spelling
> **azar** in lowercase mono, just pressed into thick cotton paper, deep
> crisp emboss visible in raking light, ink still slightly wet and
> glossy on the raised letters, paper fibers crushed around the
> impression, one letter (the final r) struck slightly deeper than the
> rest — imperfect, human. Single hard key light, museum conservation
> photography. Mood: the word as minted object.

## A5. Materia — el wordmark renderizado en materia (elegir una)
> a) The word **azar** as massive extruded metal letters, raw machined
>    aluminum with tooling marks, standing on dark studio floor, fog,
>    single top light. Industrial, permanent.
> b) The word **azar** carved deep into a block of dark marble, roman
>    square proportions but monospace rhythm, dust in the grooves,
>    raking light. 2400 years old, timeless.
> c) The word **azar** as glowing glass neon tubes on black brick,
>    warm amber gas, transformer hum implied, slight flicker captured
>    mid-flicker. Nocturnal, honest.

---

# B. MASCOTA / PET (la parte simbólica)

## B1. La gata agente del caos (recomendada: internet-native,熵 en patas)
> A sleek black cat mid-pounce, one paw batting a small ivory die that
> is tumbling in mid-air, motion blur only on the die, the cat in razor
> focus with huge amplitude pupils, on a dark desk beside a keyboard,
> single warm practical light. Studio pet photography, 85mm, shallow
> depth of field. Mood: entropy has fur. The die shows a five.

## B2. El coyote tramposo (identidad mexicana, hermano del águila o sol)
> A desert coyote sitting perfectly still at dusk among creosote and
> ocotillo, head slightly tilted, one eye catching the last sun, a
> tarnished gold coin balanced on its nose, thorny shadows long on the
> sand. Wildlife photography, 400mm, National Geographic tone. Mood:
> the trickster deciding nothing, deciding everything.

## B3. El axolote estocástico (tierno, raro, mexicano, meme-capaz)
> A pale pink axolotl floating in dark still water, external gills
> flared like a crown, tiny glass dice resting on the water surface
> around it with soft reflections, single overhead light shaft, black
> background. Aquarium macro photography. Mood: small, strange,
> unbothered — randomness incarnate, cute.

## B4. El pulpo de la bola ocho (nodo con bola8)
> A small octopus in an ink-dark tank wrapping two arms around a classic
> black 8-ball twice its size, sucker detail crisp, one siphon jet
> blowing a soft cloud, deep teal water, single diver light. Underwater
> macro. Mood: eight arms, eight ball, zero hesitation.

---

# C. ESCENA + LOGO INTEGRADO (el nombre acuñado dentro del mundo)

## C1. El volado acuña el nombre (la escena de la moneda, logo-dentro)
> High-speed macro of a golden coin frozen mid-spin over a dark felt
> table, and embossed on its face, the lowercase word **azar** in crisp
> monospace letterforms catching a blade of light, micro dust orbiting,
> the edge of the coin motion-blurred. Chiaroscuro single key light.
> The word is the hero of the frame, coin slightly tilted toward
> camera. Mood: the name being minted by chance itself.

## C2. El precinto con monograma
> Macro of dark-red sealing wax mid-stamp, the brass die lowering into
> the melt, and embossed into the cooling wax the word **azar** tiny
> and perfect, wax threads frozen mid-splash over Security guilloche
> paper. Oxblood + brass + cream. Mood: tamper-evident since the first
> letter.

## C3. El kleroterion entrega el nombre
> Museum-dark gallery, an ancient Greek stone allotment machine with a
> dense grid of slots, and one slot near the center glowing faint
> phosphor green holding a small tile etched with the lowercase word
> **azar**, long shadows, marble dust in the spotlight beam. Mood: 2400
> years of lottery, and the stone finally speaks.

## C4. Bola8 con inscripción
> Film-noir pool hall, an obsidian 8-ball on green felt shot so close
> the eight is out of frame, and lacquered onto the black surface in
> thin white monospace inlay: **azar**, rim light tracing the curve,
> Cinestill 800T halation, chalk dust in the air. Mood: ask the ball.

---

# D. ATMÓSFERA PURA (backdrops para componer el wordmark después)
Estas NO llevan texto: son la capa de fondo. Componer `azar` encima
en Figma con blending `screen`/`overlay`.

## D1. Bech32 nebula
> Pure black field with a faint constellation of tiny lowercase bech32
> glyphs scattered like stars, denser toward the lower right, subtle
> phosphor green nebula glow, heavy film grain, vignette. Nothing else.

## D2. Fósforo pulse
> Abstract black screen texture: soft bloom patches of phosphor green
> and amber light, scanline interference waves, CRT degauss wobble
> frozen, no characters at all, pure emission. For compositing.

## D3. felt & smoke
> Dark pool-hall felt under a single overhead cone, chalk dust and
> cigarette haze, deep green-black gradient, anamorphic flare hint.
> Empty stage for a wordmark.

---

## Notas de uso
- Orden sugerido de prueba: A1 (identidad) → B1 o B3 (mascota) → C1
  (social preview) → D1 como backdrop de README.
- Magnific (cuando esté el MCP): creativity ≤ 3, upscale 2x. En piezas
  A* con texto, Magnific SOLO en el fondo (enmascarar las letras) o
  riesgo de alucinación tipográfica.
- Si el modelo respeta charset: añadir `characters strictly from
  qpzry9x8gf2tvdw0s3jn54khce6mua7l` (Flux obedece mejor que MJ).
- El wordmark SIEMPRE lowercase, SIEMPRE monospace: es output válido del
  propio tool — esa es la identidad.
