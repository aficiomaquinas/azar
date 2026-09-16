# azar — brand image prompt portfolio

Workflow: generate (Midjourney/Flux/DALL-E) → Magnific (upscale, creativity
LOW for fidelity) → add the "azar" wordmark LAST in Figma/Canva (AI models
mangle spelled text; leave negative space in composition instead).

Aspect ratios: GitHub social preview 1280x640 · README banner ~1200x400 ·
logo/avatar 1:1. Universal negative: `text, letters, watermark, logo,
signature, extra fingers, jpeg artifacts`.

---

## 1. CRT terminal room (el aesthetic que pidió: código + bech32 + fósforo)
> Vintage 1980s computer terminal room at night, a single CRT monitor
> glowing phosphor green, rows of lowercase monospace characters raining
> softly down the curved glass screen, heavy scanlines and screen-door
> pixel texture, light bloom around the brightest glyphs, one lone
> typist silhouette reflected in the glass, dust motes in the beam,
> dark room lit only by the display. Shot on Kodak Portra 400, 35mm,
> shallow depth of field. Mood: focused, analog, secretive.
> Palette: black, phosphor green #3ddc84, amber accents.
> Leave the lower third of the screen as soft empty glow (wordmark space).

## 2. Águila o sol — the volado (el volado que eligió el nombre)
> Extreme high-speed photography of a golden coin spinning mid-air above
> a sun-baked desert mesa at golden hour, frozen motion, dust particles
> suspended around it, dramatic side light, sun eagle embossed on one
> face catching the light, horizon low and vast. National Geographic
> style, 300mm lens, razor-thin focal plane on the coin. Mood: fate
> suspended, decisive moment. Warm ochre + gold palette.
> The coin face is blank polished metal (no text).

## 3. La fortuna — baroque still life (los aliases: ficha, gettone, dado)
> Caravaggio-style chiaroscuro still life on a dark oak table: an
> antique brass roulette wheel section, worn ivory dice, brass payphone
> tokens and clay gaming chips scattered mid-tumble, a single beam of
> window light carving the objects out of blackness, oil-on-canvas
> texture with visible brushwork, deep umber and candle-gold palette.
> Museum macro photography of a 17th-century painting. Mood: gravity,
> old-world chance, quiet drama.

## 4. El precinto — wax seal macro (checksum = tamper-evident)
> Extreme macro photography of molten dark-red sealing wax being stamped
> with a brass monogram die, fine Security-pattern guilloche paper
> underneath, wax threads and droplets frozen mid-splash, single hard
> key light, razor micro-contrast, museum conservation photography.
> Palette: oxblood wax, aged paper cream, brass. Mood: tamper-evident,
> official, intimate. The die face is a simple abstract glyph (no letters).

## 5. Kleroterion — the Athenian lottery machine (la historia profunda)
> Museum photography of an ancient Greek stone allotment machine
> (kleroterion): a weathered marble slab with a dense grid of slots,
> small bronze tokens resting in some slots, dramatic single spotlight
> in a dark gallery, long shadows, fine marble grain, 50mm perspective.
> Mood: 2400 years of trusted randomness, civic gravity, timeless.
> Palette: Pentelic marble ivory, deep shadow, one warm spotlight.

## 6. Bola8 noir (el alias estrella)
> Film-noir pool hall at 2am, a colossally oversized obsidian magic
> 8-ball resting on felt, low-key lighting with a single overhead cone,
> cigarette-smoke haze, rim light tracing the ball's curve, the number
> eight as the only marking, green felt and black lacquer palette,
> shot on Cinestill 800T with halation. Mood: ask, shake, receive.
> Anamorphic 2.39:1 composition, ball off-center right.

## 7. Swiss type poster (la opción "logo limpio" renderizada)
> Flat Swiss International Style poster: enormous lowercase monospace
> word "azar" set in a strict grid, one signal color on off-white paper,
> generous negative space below the word, subtle paper grain and ink
> emboss texture, risograph print misregistration 1mm. No other elements
> except a tiny index line at the bottom margin. Mood: precise, quiet,
> confident. (Vector-clean render; this one CAN include the word —
> single short lowercase word usually survives; regenerate if mangled.)

## 8. Zen entropy (el contrapunto minimal)
> Wabi-sabi zen garden at dawn: raked gravel forming a perfect wave
> pattern that terminates abruptly in a single carved stone die resting
> off-grid, morning fog, moss, muted stone greys with one warm accent
> of morning light. Medium-format film photography, painterly negative
> space upper half. Mood: randomness accepted, stillness. Minimal,
> contemplative.

---

## Notas de uso
- Para banners CON texto: usar #7 (o componer texto después sobre #1-#6).
- Magnific: preset "Sparkle" o "Illusio" suave, creativity ≤ 3 para no
  inventar texto; upscale 2x basta para GitHub (1280px ya es nativo).
- El estilo CRT (#1) acepta variación: fósforo ámbar en vez de verde
  (más distintivo que el verde Matrix-cliché), o el charset bech32
  explícito en el prompt si el modelo lo respeta: "characters from the
  set qpzry9x8gf2tvdw0s3jn54khce6mua7l" (funciona en Flux mejor que en MJ).
