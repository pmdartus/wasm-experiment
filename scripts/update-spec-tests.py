import glob;
import subprocess;
import os.path as path;

rootDir = path.dirname(path.dirname(path.abspath(__file__)))
specDir = path.join(rootDir, "spec")
specTestDir = path.join(specDir, "test/core")

fixtureDir = path.join(rootDir, "tests/fixtures")

wabtGitRemote = "git@github.com:WebAssembly/wabt.git"
wabtInstallDir = "/tmp/wabt"
wast2jsonLocation = path.join(wabtInstallDir, "bin/wast2json")

def update_spec():
    # Cleanup and pull down an up to date version of the spec
    subprocess.run(["rm", "-rf", "spec"])
    subprocess.run(["git", "submodule", "update", "--init"])
    subprocess.run(["git", "checkout", "master"], cwd=specDir)
    subprocess.run(["git", "pull"], cwd=specDir)

def build_tests():
    # Clean up existing fixture directory
    subprocess.run(["rm", "-rf", fixtureDir])
    subprocess.run(["mkdir", fixtureDir])

    # Clone and build wabt only if necessary
    if not path.isdir(wabtInstallDir):
        subprocess.run(["rm", "-rf", wabtInstallDir])
        subprocess.run(["git", "clone", "--recursive", wabtGitRemote, wabtInstallDir])
        subprocess.run(["make"], cwd=wabtInstallDir)

    # Convert the spec "wast" file to standard "wasm" modules using "wast2json"
    for wastFile in glob.glob(specTestDir + "/*.wast"):
        wastFileName = path.basename(wastFile)
        moduleFileName = wastFileName.replace(".wast", ".json")
        moduleLocation = path.join(fixtureDir, moduleFileName)
        subprocess.call([wast2jsonLocation, wastFile, "-o", moduleLocation])

if __name__ == "__main__":
    update_spec()
    build_tests()