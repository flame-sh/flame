# Copyright 2023 The xflops Authors.
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

area = 0.0


def get_summary(task):
    global area
    if task.state == flame.TaskState.Succeed:
        area += float(task.output)


conn = flame.connect("127.0.0.1:8080")
ssn = conn.create_session(application="pi", slots=1)

task_inputs = ["100000"] * 1000
ssn.run_all_tasks(task_inputs, get_summary)
pi = 4 * area / (100000 * 1000)
print(pi)

ssn.close()

