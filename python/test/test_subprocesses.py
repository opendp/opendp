import subprocess

def test_subprocess():
    subprocess.run('cd .. && flake8 . --count --show-source --statistics', shell=True, check=True)
