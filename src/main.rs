use yoru::{Element, div, Application, Root, Sizing, Color, Label};

const EXAMPLE_TEXT: &'static str = r"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec tincidunt nunc lacus, nec finibus dolor sollicitudin tristique. Suspendisse sed magna sed felis fringilla tempus vel sit amet arcu. Praesent quis quam a nibh pretium blandit. Phasellus viverra nunc tempus ullamcorper euismod. Curabitur consequat posuere dolor, vitae auctor velit viverra eget. Nullam pellentesque rutrum enim, vitae congue nunc lacinia blandit. Nullam at nibh lacus. Suspendisse purus neque, venenatis at pulvinar sit amet, semper eu tortor. Nulla facilisi. Interdum et malesuada fames ac ante ipsum primis in faucibus.

Vestibulum id aliquam magna. Nullam tristique consequat luctus. Proin sodales eu est ut efficitur. Donec pulvinar sed massa id bibendum. Aliquam erat volutpat. Nulla ac porttitor nibh, id dignissim enim. Aenean sed congue nunc. Cras ac pulvinar arcu. Praesent ultricies volutpat est non tempor. Mauris luctus orci nec purus aliquam malesuada. Sed mi enim, gravida sit amet arcu et, egestas convallis risus.

Donec volutpat sapien id justo rhoncus, id maximus magna blandit. Vestibulum ac suscipit nisi. Morbi sit amet magna magna. Fusce consequat lorem eu lectus luctus interdum. Sed quam mauris, vehicula nec blandit ut, ornare eget nulla. Nulla bibendum vulputate leo, id rhoncus erat vulputate quis. Aliquam erat volutpat. Sed accumsan consequat lorem eu vehicula. Vestibulum aliquet lectus vel lacus rutrum iaculis. Pellentesque augue nisi, feugiat et nunc at, condimentum ultricies mi. Integer lacinia, justo congue aliquet bibendum, nunc felis fringilla augue, sit amet malesuada odio nunc sed neque. Proin non mi commodo nulla mollis lacinia vel sed sapien.

Phasellus sit amet scelerisque nulla. Sed ante metus, rhoncus et elit non, bibendum lacinia dui. Integer non efficitur nibh, in faucibus leo. Aenean quis scelerisque purus. Etiam scelerisque, nunc luctus rutrum vehicula, orci magna facilisis nibh, eu vulputate neque ipsum eget quam. Phasellus sit amet augue purus. Morbi ut ex quis neque ornare scelerisque.

Aenean porta iaculis eleifend. Nam pulvinar quis sapien ut congue. Suspendisse ut malesuada mauris, faucibus sollicitudin magna. Fusce ac dui eu elit consectetur ultrices. Curabitur consectetur elementum imperdiet. Ut maximus neque elit, vitae hendrerit purus laoreet ut. Sed hendrerit pellentesque rutrum. Etiam iaculis sem nec lorem placerat, rhoncus scelerisque lectus scelerisque. Aliquam suscipit vel nunc sed efficitur. Praesent tempor erat velit, sed ornare tellus finibus nec. Nulla eget metus erat. Mauris non porta lectus, nec vestibulum arcu. Nam sem ante, pretium ut ex vel, venenatis pretium ligula.";


fn main() {
    let model = 7;

    let b: Element<i32> = div!(width=Sizing::Fit, margin=10.0, background=Color::LIGHT_GRAY, [
        div!(width=Sizing::Expand, height=Sizing::Fixed(10.0)),
        Label::new(|_| EXAMPLE_TEXT.into()),
        Label::new(|app: &mut i32| app.to_string())
    ]).into();

    Application::new(model, Root::new(b)).run();
}
