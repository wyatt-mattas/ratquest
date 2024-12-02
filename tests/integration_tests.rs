use ratquest::app::{ApiRequest, App, AuthDetails, AuthType, BasicAuth, RequestType};

mod app_integration_tests {
    use super::*;

    #[test]
    fn test_full_request_workflow() {
        let mut app = App::new();

        // Create a new group
        app.key_input = String::from("test_group");
        app.save_group();

        // Verify group was created
        assert!(app.list.contains_key("test_group"));

        // Add a request to the group
        app.selected_group = Some("test_group".to_string());
        app.request_name_input = String::from("test_request");
        app.selected_request_type = RequestType::POST;
        app.save_request();

        // Verify request was created
        let requests = app.list.get("test_group").unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].name, "test_request");
        assert!(matches!(requests[0].request_type, RequestType::POST));

        // Set up details for the request
        app.selected_group_index = Some(0);
        app.selected_request_index = Some(0);

        // Add URL
        app.url_textarea =
            tui_textarea::TextArea::from(vec![String::from("http://api.example.com")]);
        app.save_textarea_content();

        // Add Basic Auth
        if let Some(request) = app.get_selected_request_mut() {
            request.details.auth_type = AuthType::Basic;
            request.details.auth_details = AuthDetails::Basic(BasicAuth {
                username: String::from("testuser"),
                password: String::from("testpass"),
            });
        }

        app.auth_username_textarea = tui_textarea::TextArea::from(vec![String::from("testuser")]);
        app.auth_password_textarea = tui_textarea::TextArea::from(vec![String::from("testpass")]);
        app.save_textarea_content();

        // Verify final state
        if let Some(request) = app.get_selected_request() {
            assert_eq!(request.details.url, "http://api.example.com");
            assert!(request.details.headers.contains_key("Authorization"));
            if let AuthDetails::Basic(auth) = &request.details.auth_details {
                assert_eq!(auth.username, "testuser");
                assert_eq!(auth.password, "testpass");
            } else {
                panic!("Expected Basic auth details");
            }
        } else {
            panic!("Failed to get selected request");
        }
    }
}

mod tree_integration_tests {
    use super::*;

    // #[test]
    // fn test_tree_navigation() {
    //     let mut app = App::new();

    //     // Set up test data
    //     app.list.insert(
    //         "group1".to_string(),
    //         vec![
    //             ApiRequest::new("request1".to_string(), RequestType::GET),
    //             ApiRequest::new("request2".to_string(), RequestType::POST),
    //         ],
    //     );

    //     app.update_groups_vec();

    //     // Build initial tree and verify its structure
    //     app.build_tree();

    //     // Print the structure of our tree for debugging
    //     println!("Initial tree state:");
    //     println!("Selected node: {:?}", app.tree_state.selected());

    //     // First move - should go to first group
    //     app.tree_next();
    //     println!("After first tree_next:");
    //     println!("Selected node: {:?}", app.tree_state.selected());

    //     // Try toggling the group open
    //     app.tree_toggle();
    //     println!("After toggle:");
    //     println!("Selected node: {:?}", app.tree_state.selected());

    //     // Navigate to the first request
    //     app.tree_next();
    //     println!("After navigating to first request:");
    //     println!("Selected node: {:?}", app.tree_state.selected());

    //     // Verify final state
    //     if let Some(selected) = app.tree_state.selected() {
    //         assert!(
    //             selected.contains("request1"),
    //             "Expected to be on request1, but got: {}",
    //             selected
    //         );
    //     } else {
    //         panic!("No node selected!");
    //     }
    // }

    #[test]
    fn test_tree_structure() {
        let mut app = App::new();

        // Add some test data
        app.list.insert(
            "group1".to_string(),
            vec![
                ApiRequest::new("request1".to_string(), RequestType::GET),
                ApiRequest::new("request2".to_string(), RequestType::POST),
            ],
        );

        app.update_groups_vec();

        // Build the tree
        let tree = app.build_tree();
        let root = tree.root();

        // Verify root structure
        assert_eq!(root.id(), "/");

        // Check if root has children
        let root_children: Vec<_> = root.iter().collect();
        assert_eq!(root_children.len(), 1, "Root should have 1 group");

        // Verify group structure
        let group_node = &root_children[0];
        assert!(
            group_node.id().starts_with("group-"),
            "Group ID should start with 'group-', got: {}",
            group_node.id()
        );

        // Verify group's children
        let group_children: Vec<_> = group_node.iter().collect();
        assert_eq!(group_children.len(), 2, "Group should have 2 requests");

        // Verify request nodes
        assert!(
            group_children[0].id().contains("request1"),
            "First request should contain 'request1', got: {}",
            group_children[0].id()
        );
        assert!(
            group_children[1].id().contains("request2"),
            "Second request should contain 'request2', got: {}",
            group_children[1].id()
        );
    }
}
