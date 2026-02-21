from datetime import datetime


def get_week(d=datetime.today()):
    return d.isocalendar()[1]
