from core.courses import Courses as Courses
from lesson_manager import config
import utils


def main():
    courses = Courses()
    current = courses.current
    rofi_options = config.rofi_options

    if not courses:
        utils.rofi.msg("No courses found!", err=True)
        exit(1)

    if config.highlight_current_course:
        try:
            current_index = courses.index(current)
            rofi_options.extend(["-a", current_index])
        except ValueError:
            pass

    longest_name = max(len(course.info["title"]) for course in courses)
    options = []
    for course in courses:
        title = course.info['title']
        short = course.info['short']

        padded_title = title.ljust(longest_name)

        column_1 = f"<b>{padded_title}</b>"
        column_2 = f"<i><span size='smaller'>({short})</span></i>"

        options.append(f"{column_1}  {column_2}")

    _, index, _ = utils.rofi.select(
        "Select course",
        options,
        rofi_options,
    )

    if index >= 0:
        courses.current = courses[index]


if __name__ == "__main__":
    main()
