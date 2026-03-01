import urllib.request
import os

assets_dir = "/Users/josh/Projects/lab/matlab/applo11_simulation/apollo11_bevy/assets"
os.makedirs(assets_dir, exist_ok=True)

# 4K Texture URLs from Celestia / NASA SVS archives
textures = {
    # Earth
    "earth_4k.jpg": "https://www.solarsystemscope.com/textures/download/8k_earth_daymap.jpg", # 8k mapped but highly compressed
    "earth_normal_4k.jpg": "https://www.solarsystemscope.com/textures/download/8k_earth_normal_map.jpg",
    "earth_specular_4k.jpg": "https://www.solarsystemscope.com/textures/download/8k_earth_specular_map.jpg",
    "earth_clouds_4k.jpg": "https://www.solarsystemscope.com/textures/download/8k_earth_clouds.jpg",
    "earth_night_4k.jpg": "https://www.solarsystemscope.com/textures/download/8k_earth_nightmap.jpg",
    
    # Moon
    "moon_4k.jpg": "https://www.solarsystemscope.com/textures/download/8k_moon.jpg",
    
    # Sun
    "sun_4k.jpg": "https://www.solarsystemscope.com/textures/download/8k_sun.jpg",
    
    # Starfield Backup
    "stars_4k.jpg": "https://www.solarsystemscope.com/textures/download/8k_stars_milky_way.jpg"
}

print("Downloading Hyper-Detailed Textures...")
for filename, url in textures.items():
    filepath = os.path.join(assets_dir, filename)
    if not os.path.exists(filepath):
        print(f"Downloading {filename}...")
        try:
            req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
            with urllib.request.urlopen(req) as response, open(filepath, 'wb') as out_file:
                out_file.write(response.read())
            print(f"  -> Saved {filename}")
        except Exception as e:
            print(f"  -> Failed to download {filename}: {e}")
    else:
        print(f"Skipping {filename} (Already exists)")

print("Download Complete!")
