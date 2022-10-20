# data passed between github actions steps needs to be encoded in a particular way

# mash all arguments into a single "files" string
files="$*"

# construct list of .tex files
tex="${files//'%'/'%25'}"
tex="${tex//$'\n'/'%0A'}"
tex="${tex//$'\r'/'%0D'}"
echo "tex=$tex" >> $GITHUB_OUTPUT

# construct list of .pdf files
pdf="${files//$'.tex'/.pdf}"
pdf="${pdf//'%'/'%25'}"
pdf="${pdf//$'\n'/'%0A'}"
pdf="${pdf//$'\r'/'%0D'}"
echo "pdf=$pdf" >> $GITHUB_OUTPUT