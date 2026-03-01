import urllib.request
import os

assets_dir = "/Users/josh/Projects/lab/matlab/applo11_simulation/apollo11_bevy/assets"
textures = {
    # Using reliable fallback mirrors for the maps that failed
    "earth_normal_4k.jpg": "https://upload.wikimedia.org/wikipedia/commons/thumb/c/c9/Earth_Normal_Map_-_8k.jpg/2560px-Earth_Normal_Map_-_8k.jpg",
    "earth_specular_4k.jpg": "https://upload.wikimedia.org/wikipedia/commons/4/4b/Earth_Specular_Map.jpg"
}

print("Downloading missing Normal/Specular textures...")
for filename, url in textures.items():
    filepath = os.path.join(assets_dir, filename)
    print(f"Downloading {filename}...")
    try:
        req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
        with urllib.request.urlopen(req) as response, open(filepath, 'wb') as out_file:
            out_file.write(response.read())
        print(f"  -> Saved {filename}")
    except Exception as e:
        print(f"  -> Failed: {e}")
