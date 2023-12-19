gci $psScriptroot\ cargo.toml -recurse | % {
    $dir = $_ | Split-Path -parent
    write-host $dir
    cd $dir
    cargo clean
}