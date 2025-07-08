INDEX=	https://download.FreeBSD.org/ports/index/INDEX-14.bz2

all:
	curl -s ${INDEX} | bunzip2 | egrep '^(ar-aspell)-[0-9]' > ok
