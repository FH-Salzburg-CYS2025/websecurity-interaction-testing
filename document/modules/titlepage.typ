#let present(x) = x != none and x.trim().len() > 0

#let centered-heading(settings, size, weight, body) = align(center, text(
  font: settings.font-heading,
  size,
  weight: weight,
  body,
))

#let meta-row(label, value) = (strong(label), value)

#let meta-block(
  author,
  head-department,
  supervisor,
  submission-date,
  size: 1.1em,
) = {
  let cells = (
    meta-row("Author: ", author),
    if present(head-department) {
      meta-row("Head of Department: ", head-department)
    } else { () },
    meta-row("Supervisor: ", supervisor),
    if submission-date != none {
      meta-row("Submission Date: ", submission-date)
    } else { () },
  ).flatten()

  pad(
    top: 3em,
    right: 10%,
    left: 10%,
    {
      set text(size: size)
      grid(
        columns: (50% - 0.5em, 1fr),
        gutter: 1em,
        ..cells,
      )
    },
  )
}

#let open-title-page(settings: ()) = {
  set page(
    paper: "a4",
    margin: (left: 30mm, right: 30mm, top: 40mm, bottom: 40mm),
    numbering: "I",
    number-align: center,
    footer: "",
  )

  set text(font: settings.font-body, size: settings.font-body-size, lang: "en")
  set par(leading: 1em)
}

#let finish-title-page(
  settings: (),
  degree: "",
  title: "",
  subtitle: "",
  title-german: "",
  subtitle-german: "",
  author: "",
  head-department: "",
  supervisor: "",
  submission-date: none,
) = {
  let gap-top = 15mm
  let gap-after-degree = 8mm
  let gap-between-langs = 10mm

  v(gap-top)

  centered-heading(
    settings,
    1.5em,
    100,
    "Attack-Defense-Simulation 1\nlaboratory protocol",
  )
  v(gap-after-degree)

  let print-title-block = (t, st, title-size, subtitle-size) => {
    centered-heading(settings, title-size, 700, t)
    if present(st) { centered-heading(settings, subtitle-size, 500, st) }
  }

  if present(title-german) {
    if present(subtitle) or present(subtitle-german) {
      print-title-block(title, subtitle, 1.2em, 1.2em)
      v(gap-between-langs)
      print-title-block(title-german, subtitle-german, 1.2em, 1.2em)
    } else {
      print-title-block(title, none, 1.4em, 1.0em)
      v(gap-between-langs)
      print-title-block(title-german, none, 1.4em, 1.0em)
    }
  } else {
    if present(subtitle) {
      print-title-block(title, subtitle, 1.8em, 1.4em)
    } else {
      print-title-block(title, none, 2.0em, 1.0em)
    }
  }

  meta-block(
    author,
    head-department,
    supervisor,
    submission-date,
    size: 1.1em,
  )

  pagebreak()
}
