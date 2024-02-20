# Copyright 2023 The Flame Authors.
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#     http://www.apache.org/licenses/LICENSE-2.0
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import flame

import argparse


parser = argparse.ArgumentParser(description='Flame Pi Python Example.')
parser.add_argument('-n', '--task_num', type=int, help="The total number of tasks in the session.")
parser.add_argument('-i', '--task_input', type=int, help="The input of each task to calculate Pi.")
args = parser.parse_args()


# Init the Flame environment.
flame.init()

# Define the Pi service.
@flame.service
def pi(n):
    # TODO(k82cn): wrap it in flame.service.
    import random
    import math

    sum = 0.0

    for i in range(n):
        x, y = random.uniform(0, 1.0), random.uniform(0, 1.0)
        if math.hypot(x, y) <= 1.0:
            sum += 1

    return sum


# Got result of all tasks.
area = 0.0

for i in range(args.task_num):
    area += pi(args.task_input).get()

# Calculate the Pi.
pi = 4 * area / (args.task_input * args.task_num)

print("pi = 4*({}/{}) = {}".format(area, args.task_input * args.task_num, pi))

