#!/bin/sh -e

pattern='gen_*.rs'

rs_file=`echo $0 | sed 's,\.rs\.sh$,.rs,'`

# Rust file must exist.
test -f $rs_file

dir=`dirname $rs_file`
orig=`basename $rs_file`
tmp=$orig.tmp

cd $dir

# Get rust code before "// START MOD LIST", including this start comment.
cat $orig | awk '{ if (!end) { print $0 } } /^ *\/\/ START MOD LIST/{ end = 1 }' > $tmp

# If comment is not found, than files is same. So it must be different.
sh -ec "diff -q $orig $tmp || true" | grep "differ"

# Continue if start pattern was found

# Get rs files by pattern, sort it for compatibility with rustfmt, produce mod with path.
find .. -name "$pattern" | sort |\
    sed -E 's,^\.\./gen_([a-z]+)_(0x[0-9a-f]+)_(.+).rs$,    #[path = "\0"]\n    mod gen_\1_\2_\3;\n    pub use gen_\1_\2_\3 as gen_\3;\n    use gen_\3 as \3;\n    const BYTECODE_\3: u8 = \2;,' |\
    sed -E 's,mod ([^/]*)/(.*),mod \1__\2,' |\
    sed -E 's,mod ([^/]*)/(.*),mod \1__\2,' |\
    sed -E 's,mod ([^/]*)/(.*),mod \1__\2,' |\
    cat >> $tmp

# Final part to make new file, append code after end marker, including marker.
cat $orig | awk '/^ *\/\/ END MOD LIST/{ start = 1 } { if (start) { print $0 } }' >> $tmp

# Checking that new file is ok, else removing tmp file.
sh -ec "diff -u $orig $tmp | grep '^\+ *mod [a-z]'" || sh -c "rm -f $tmp; false"

# Swapping files
chmod +x $tmp
mv $tmp $orig

echo Commited
