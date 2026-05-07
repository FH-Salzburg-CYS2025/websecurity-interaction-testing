#import "../lib.typ": *
#import "abbreviations.typ": abbreviations
#import "settings/metadata.typ": *
#import "settings/settings.typ": *
#import "@preview/codly:1.3.0": *
#import "@preview/codly-languages:0.1.10": *

#set document(title: title-english, author: author)
#set cite(style: settings.citation-style)

// Glossary setup (global)
#show: make-glossary
#register-glossary(abbreviations)

#open-title-page(settings: settings)

#align(center)[
  #image("../assets/fhs_logo.svg", width: 55%)
]

#finish-title-page(
  settings: settings,
  degree: degree,
  title: title-english,
  subtitle: subtitle-english,
  author: author,
  supervisor: supervisor,
  submission-date: submission-date,
)

#show: preface.with(settings: settings)
#listings(abbreviations: abbreviations)

#show: main-body.with(settings: settings)
#show: codly-init.with()
#codly(languages: (
  ..codly-languages,
  ipm: (
    name: [IPM],
    color: rgb("#5b8dd9"),
    icon: [⚙],
  ),
))
#codly-enable()
#show raw.where(lang: "ipm"): set raw(
  syntaxes: "ipm.sublime-syntax",
)
#include "chapters/include.typ"
