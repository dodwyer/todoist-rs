use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;
use reqwest::Client;
use tui_textarea::TextArea;

use crate::{
    api_calls::{self, close_task},
    task_edit, App,
};

pub fn handle_task_editor(app: &mut App, key: KeyEvent, client: Client) {
    if key.code == KeyCode::Esc {
        app.show_task_editor = !app.show_task_editor;
    } else if key.code == KeyCode::Enter {
        app.show_task_editor = !app.show_task_editor;
        let index = app.task_edit.current_task_index;

        app.tasks.tasks[index].content = app.task_edit.content.lines().join("\n");
        app.tasks.tasks[index].description = app.task_edit.description.lines().join("\n");
        let task = app.tasks.tasks[index].clone();
        tokio::spawn(async move {
            let _ = api_calls::update_task(&client, task).await;
        });
    }
    if key.code == KeyCode::Tab {
        if app.task_edit.currently_editing == task_edit::CurrentlyEditing::Content {
            app.task_edit.currently_editing = task_edit::CurrentlyEditing::Description
        } else if app.task_edit.currently_editing == task_edit::CurrentlyEditing::Description {
            app.task_edit.currently_editing = task_edit::CurrentlyEditing::ChildTasks
        } else {
            app.task_edit.currently_editing = task_edit::CurrentlyEditing::Content
        }
        return;
    }

    if app.task_edit.currently_editing == task_edit::CurrentlyEditing::Content {
        app.task_edit.content.input(key);
    } else if app.task_edit.currently_editing == task_edit::CurrentlyEditing::Description {
        app.task_edit.description.input(key);
    } else {
        if key.code == KeyCode::Char('j') {
            app.task_edit.next();
        } else if key.code == KeyCode::Char('k') {
            app.task_edit.previous();
        } else if key.code == KeyCode::Enter {
            if let Some(selected) = app.task_edit.children_list_state.selected() {
                app.show_task_editor = true;
                let index = app.task_edit.children[selected];
                let selected = &app.tasks.tasks[index];

                let mut children = Vec::new();

                for (index, task) in app.tasks.tasks.iter().enumerate() {
                    if task.parent_id == Some(selected.id.clone()) {
                        children.push(index);
                    }
                }

                app.task_edit = task_edit::TaskEdit {
                    content: TextArea::from(vec![selected.content.clone()]),
                    description: TextArea::from(vec![selected.description.clone()]),
                    currently_editing: task_edit::CurrentlyEditing::Content,
                    children: children,
                    children_list_state: ListState::default(),
                    current_task_index: index,
                }
            }
        }
    }
}

pub fn handle_projects(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Char('j') {
        app.projects.next();
    } else if key.code == KeyCode::Char('k') {
        app.projects.previous();
    } else if key.code == KeyCode::Enter {
        if let Some(selected) = app.projects.state.selected() {
            let selected_id = app.projects.projects[selected].id.clone();
            app.tasks.filter = crate::tasks::Filter::ProjectId(selected_id.clone());
            app.tasks.filter_task_list();
            app.projects.selected_project = Some(selected_id);
        }
    } else if key.code == KeyCode::Char('x') {
        todo!("DELETE PROJECT");
    } else if key.code == KeyCode::Char('+') {
        if let Some(selected) = app.projects.state.selected() {
            let selected_id = app.projects.projects[selected].id.clone();
            app.show_new_task = true;
        }
    }
}

pub fn handle_tasks(app: &mut App, key: KeyEvent, client: Client) {
    if key.code == KeyCode::Char('j') {
        app.tasks.next();
    } else if key.code == KeyCode::Char('k') {
        app.tasks.previous();
    } else if key.code == KeyCode::Enter {
        if let Some(selected) = app.tasks.state.selected() {
            app.show_task_editor = true;
            let index = app.tasks.display_tasks[selected];
            let selected = &app.tasks.tasks[index];

            let mut children = Vec::new();

            for (index, task) in app.tasks.tasks.iter().enumerate() {
                if task.parent_id == Some(selected.id.clone()) {
                    children.push(index);
                }
            }

            app.task_edit = task_edit::TaskEdit {
                content: TextArea::from(vec![selected.content.clone()]),
                description: TextArea::from(vec![selected.description.clone()]),
                currently_editing: task_edit::CurrentlyEditing::Content,
                children: children,
                children_list_state: ListState::default(),
                current_task_index: index,
            }
        }
    } else if key.code == KeyCode::Char('x') {
        if let Some(selected) = app.tasks.state.selected() {
            let index = app.tasks.display_tasks[selected];
            let task_id = app.tasks.tasks[index].id.clone();
            app.tasks.state = ListState::default();
            app.tasks.display_tasks.remove(selected);
            app.tasks.tasks.remove(index);
            app.tasks.filter_task_list();
            tokio::spawn(async move {
                close_task(&client, task_id).await.unwrap();
            });
        }
    }
}