#import "modules/titlepage.typ": *
#import "@preview/glossarium:0.5.10": (
  gls as _gls, glspl as _glspl, make-glossary as _make-glossary, print-glossary,
  register-glossary as _register-glossary,
)

#let in-outline = state("in-outline", false)

#let gls = _gls
#let glspl = _glspl
#let make-glossary = _make-glossary
#let register-glossary = _register-glossary

#let flex-caption(long, short) = context {
  if in-outline.at(here()) { short } else { long }
}

#let reset_figure_counters() = {
  counter(figure.where(kind: table)).update(0)
  counter(figure.where(kind: image)).update(0)
  counter(figure.where(kind: raw)).update(0)
}

#let heading_sel = selector(heading)

// Returns the last heading before a given location.
// If you want only chapters for headers, pass only_depth_1: true.
#let last_heading_before(loc, only_depth_1: false) = {
  if only_depth_1 {
    query(selector(heading.where(depth: 1)).before(loc)).last()
  } else {
    query(heading_sel.before(loc)).last()
  }
}

// Computes displayed numbering for a heading element at its location,
// falling back to the global headings-numbering-style if the element has none.
#let display_heading_number(el, settings) = {
  if el == none { return none }
  let nums = counter(heading).at(el.location())

  if el.has("numbering") and el.numbering != none {
    numbering(el.numbering, ..nums)
  } else {
    numbering(settings.headings-numbering-style, ..nums)
  }
}

#let preface(
  settings: (),
  preface,
) = {
  // Page Setup
  set page(
    margin: (
      left: settings.page-margins.left,
      right: settings.page-margins.right,
      top: settings.page-margins.top,
      bottom: settings.page-margins.bottom,
    ),
    numbering: "I",
    number-align: center,
  )
  counter(page).update(2)

  // Body Font Family
  set text(font: settings.font-body, size: settings.font-body-size, lang: "en")
  show math.equation: set text(weight: 400)

  // Headings
  show heading: set block(
    below: settings.headings-spacing.below,
    above: settings.headings-spacing.above,
  )
  show heading: set text(
    font: settings.font-body,
    size: settings.font-heading-size,
  )
  set heading(numbering: none)

  // Paragraphs
  set par(leading: settings.distance-between-lines, justify: true)

  // Figures
  show figure: set text(size: settings.font-figures-subtitle-size)

  // Indentation of Lists
  set list(indent: settings.list-indentation)
  set enum(indent: settings.list-indentation)

  preface
}

#let listings(
  abbreviations: (),
) = {
  // Enable short captions to omit citations inside outlines.
  show outline: it => {
    in-outline.update(true)
    it
    in-outline.update(false)
  }

  // Inhaltsverzeichnis
  outline(
    title: { heading(outlined: false, "Inhaltsverzeichnis") },
    target: heading.where(supplement: [Chapter], outlined: true),
    indent: auto,
    depth: 3,
  )
  v(2.4fr)
  pagebreak()
  // Abbildungsverzeichnis
  outline(
    title: { heading(outlined: false, "Abbildungsverzeichnis") },
    target: figure.where(kind: image),
  )
  pagebreak()
  // Abkürzungsverzeichnis
  heading(outlined: false)[Abkürzungsverzeichnis]
  print-glossary(abbreviations, show-all: false, disable-back-references: true)
}

#let main-body(
  settings: (),
  body,
) = {
  // Main Body numbering
  set heading(
    numbering: settings.headings-numbering-style,
    supplement: [Chapter],
  )

  // Chapter starts -> pagebreak (unless unnumbered), and reset figure counters
  show heading.where(level: 1): it => {
    if it.numbering == none {
      [#it]
    } else {
      [#pagebreak() #it]
    }
    reset_figure_counters()
  }

  // Figure numbering: <chapter>.<figure>
  set figure(numbering: it => {
    let h = counter(heading).display()
    let dot = h.position(".")
    let top = if dot == none { h } else { h.slice(0, dot) }
    [#top.#it]
  })

  // Header + Footer
  set page(
    header: context {
      // If a new chapter starts on this page, suppress header.
      let upcoming = query(selector(heading.where(depth: 1)).after(here()))
      if (
        upcoming != () and upcoming.first().location().page() == here().page() and upcoming.first().depth == 1
      ) {
        return
      }

      // Use last heading before here() (any level), but display numbering consistently.
      let el = last_heading_before(here())
      if el == none { return }

      let num = display_heading_number(el, settings)
      let title = el.body

      // Hide header on Bibliography pages.
      if title != [Bibliography] {
        align(center, num + " " + title)
        line(length: 100%, stroke: (paint: gray))
      }
    },

    footer: context {
      let current-page = counter(page).display()
      let final-page = counter(page).final().first()

      line(length: 100%, stroke: (paint: gray))
      align(center)[#current-page / #final-page]
    },
  )

  // Switch to Arabic numbering for main content and reset page counter to 1
  set page(numbering: "1/1", number-align: center)
  counter(page).update(1)

  // Apply paragraph spacing for actual content only (keeps your intent)
  set par(spacing: settings.space-before-paragraph, justify: true)

  // Actual Content
  body
}

#let appendix(body) = {
  pagebreak()

  outline(
    title: { heading("Appendix", outlined: true, numbering: none) },
    target: heading.where(supplement: [Appendix], outlined: true),
    indent: auto,
    depth: 3,
  )

  counter(heading).update(0)

  set heading(numbering: "A.1.", supplement: [Appendix])

  show heading: it => {
    let prefixed
    if it.level == 1 and it.numbering != none {
      prefixed = [#it.supplement #counter(heading).display()]
    } else if it.numbering != none {
      prefixed = [#counter(heading).display()]
    } else {
      prefixed = []
    }

    block(below: 0.85em, above: 1.75em)[
      #prefixed #it.body
    ]
  }

  // Figure numbering in appendix: <AppendixLetter>.<figure>
  // If top-level is already a single letter (A, B, ...), keep it.
  // Otherwise, try to convert int -> letter via numbering("A", n).
  set figure(numbering: it => {
    let h = counter(heading).display()
    let dot = h.position(".")
    let top = if dot == none { h } else { h.slice(0, dot) }

    let letter = if top.len() == 1 { top } else { numbering("A", int(top)) }

    [#letter.#it]
  })

  // Same chapter-start behavior as main-body + counter resets
  show heading.where(level: 1): it => {
    if it.numbering == none {
      [#it]
    } else {
      [#pagebreak() #it]
    }
    reset_figure_counters()
  }

  body
}

#let todo(body) = [
  #let rblock = block.with(stroke: red, radius: 0.5em, fill: red.lighten(80%))
  #let top-left = place.with(top + left, dx: 1em, dy: -0.35em)
  #block(inset: (top: 0.35em), {
    rblock(width: 100%, inset: 1em, body)
    top-left(rblock(fill: white, outset: 0.25em, text(fill: red)[*TODO*]))
  })
  <todo>
]

#let outline-todos(title: [TODOS]) = {
  heading(numbering: none, outlined: false, title)

  context {
    let todos = query(<todo>)
    let groups = ()
    let last = none

    for td in todos {
      let hd = query(heading_sel.before(td.location())).last()
      if last != hd {
        groups.push((heading: hd, todos: (td,)))
        last = hd
      } else {
        groups.last().todos.push(td)
      }
    }

    for g in groups {
      // Heading line with dotted fill + page number
      link(g.heading.location())[
        #display_heading_number(g.heading, (headings-numbering-style: "1.1."))
        #g.heading.body
      ]
      [ ]
      box(width: 1fr, repeat[.])
      [ ]
      [#g.heading.location().page()]

      linebreak()

      // Indented list of todos under the heading
      pad(
        left: 1em,
        g
          .todos
          .map(td => list.item(link(
            td.location(),
            td.body.children.at(0).body,
          )))
          .join(),
      )
    }
  }
}
