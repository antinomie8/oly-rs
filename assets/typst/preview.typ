${packages}

#set text(lang: "${language}")

#let has_title = (str.len("${title}") != 0)
#let subtitle = if (str.len("${subtitle}") != 0) { "${subtitle}" } else { none }
#let date = if str.len("${date}") == 0 {
	datetime.today().display("[day] [month repr:long] [year]")
} else { "${date}" }

#show: setup.with(
	title: if has_title { "${title}" } else { "${source}" },
	subtitle: subtitle,
	author: "${author}",
	date: date,
	maketitle: has_title,
)

// main content
#include "solution.typ"
