TEX = pdflatex
BIB = bibtex
MAIN = voronoid
BIBFILE = references.bib

.PHONY: all clean

all: $(MAIN).pdf

$(MAIN).pdf: $(MAIN).tex $(BIBFILE)
	$(TEX) $(MAIN)
	$(BIB) $(MAIN)
	$(TEX) $(MAIN)
	$(TEX) $(MAIN)

clean:
	rm -f *.aux *.log *.bbl *.blg *.out $(MAIN).pdf