for f in test-projects/*; do
    if [ -d "$f" ]; then
        echo "$f"

        rojo sourcemap -o "$f/sourcemap.json" "$f/default.project.json"
    fi
done
