__all__ = ["client", "service"]


import os 
import sys

script_dir = os.path.dirname(os.path.realpath(__file__))
sys.path.append(os.path.join(script_dir, "."))
sys.path.append(os.path.join(script_dir, "rpc"))

