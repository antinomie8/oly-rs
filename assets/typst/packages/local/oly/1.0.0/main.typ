// main setup
#let setup(
	title: none,
	subtitle: none,
	author: (),
	date: none,
	maketitle: true,
	body,
) = {
	if type(author) == str { author = (author,) }
	set document(title: title, author: author)

	set par(justify: true)

	body
}

#let oly(name, ..arg) = {
	let text = arg.at(0, default: name)
	link("oly://gen?name=" + name, text)
}

#let counter = 0
#let problem(body) = context [
	#{ counter = counter + 1 }
	*Problem #counter:* #body
]
#let problem(source, body) = context [
	#{ counter = counter + 1 }
	*Problem #counter (#source):* #body
]
#let _problem(source, body) = [
	*Problem (#source):* #body
]
#let _problem(source, body) = [
	*Problem:* #body
]

#let solution(body) = [
	*Solution:* #body
	#align(right, $square$)
]
